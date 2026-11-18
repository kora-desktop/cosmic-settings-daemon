// SPDX-License-Identifier: GPL-3.0-only

//! The keyboard backlight, through UPower's composite `KbdBacklight` device.
//!
//! UPower reports a maximum of 0 when no keyboard has a backlight, and then
//! nothing here writes a level.

use futures::{Stream, StreamExt};
use std::sync::atomic::{AtomicI32, Ordering};
use upower_dbus::KbdBacklightProxy;

/// At most this many presses go from off to full.
const MAX_STEPS: i32 = 10;

pub struct KbdBacklight {
    proxy: KbdBacklightProxy<'static>,
    /// The level the toggle turns the light back on at.
    last_lit: AtomicI32,
}

impl KbdBacklight {
    pub async fn new(connection: &zbus::Connection) -> zbus::Result<Self> {
        let proxy = KbdBacklightProxy::builder(connection)
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await?;
        Ok(Self {
            proxy,
            last_lit: AtomicI32::new(0),
        })
    }

    /// The current level, or `None` without a keyboard backlight.
    pub async fn brightness(&self) -> Option<i32> {
        self.max_brightness().await?;
        self.proxy.get_brightness().await.ok()
    }

    /// The highest level, or `None` without a keyboard backlight.
    pub async fn max_brightness(&self) -> Option<i32> {
        self.proxy
            .get_max_brightness()
            .await
            .ok()
            .filter(|max| *max > 0)
    }

    /// Sets the level, clamped to the device's range, and returns it.
    pub async fn set_brightness(&self, value: i32) -> Option<i32> {
        let max = self.max_brightness().await?;
        let target = value.clamp(0, max);
        self.write(target).await
    }

    /// Moves one step up (`direction > 0`) or down, and returns the new level.
    pub async fn step(&self, direction: i32) -> Option<i32> {
        let max = self.max_brightness().await?;
        let current = self.proxy.get_brightness().await.ok()?;
        let target = step_target(current, max, direction);
        if target == current {
            return Some(current);
        }
        self.write(target).await
    }

    /// Turns the light off, or back on at the level it had.
    pub async fn toggle(&self) -> Option<i32> {
        let max = self.max_brightness().await?;
        let current = self.proxy.get_brightness().await.ok()?;
        if current > 0 {
            self.last_lit.store(current, Ordering::Relaxed);
        }
        let target = toggle_target(current, max, self.last_lit.load(Ordering::Relaxed));
        self.write(target).await
    }

    /// Notes a level UPower reported, so the toggle restores levels set elsewhere.
    pub fn observe(&self, value: i32) {
        if value > 0 {
            self.last_lit.store(value, Ordering::Relaxed);
        }
    }

    /// Every level UPower reports, whoever changed it.
    pub async fn changes(&self) -> zbus::Result<impl Stream<Item = i32> + use<>> {
        let stream = self.proxy.receive_brightness_changed().await?;
        Ok(stream.filter_map(|signal| async move { signal.args().ok().map(|args| *args.value()) }))
    }

    async fn write(&self, value: i32) -> Option<i32> {
        match self.proxy.set_brightness(value).await {
            Ok(()) => {
                self.observe(value);
                Some(value)
            }
            Err(err) => {
                log::error!("Failed to set the keyboard backlight to {value}: {err}");
                None
            }
        }
    }
}

/// The next level from `current`: one of at most [`MAX_STEPS`] even steps.
pub fn step_target(current: i32, max: i32, direction: i32) -> i32 {
    if max <= 0 {
        return current;
    }
    let steps = max.min(MAX_STEPS);
    let level = |k: i32| (k * max + steps / 2) / steps;
    if direction > 0 {
        (0..=steps).map(level).find(|l| *l > current).unwrap_or(max)
    } else {
        (0..=steps)
            .rev()
            .map(level)
            .find(|l| *l < current)
            .unwrap_or(0)
    }
}

/// Off when lit; otherwise the last lit level, or full when none is known.
pub fn toggle_target(current: i32, max: i32, last_lit: i32) -> i32 {
    if current > 0 {
        0
    } else if (1..=max).contains(&last_lit) {
        last_lit
    } else {
        max
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::process::{Child, Command, Stdio};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use zbus::object_server::SignalEmitter;

    /// A dbus-daemon of the test's own, so nothing reaches this machine's buses.
    struct PrivateBus {
        daemon: Child,
        address: String,
        _dir: tempfile::TempDir,
    }

    impl PrivateBus {
        fn start() -> Self {
            let dir = tempfile::Builder::new()
                .prefix("csd-kbd")
                .tempdir_in("/tmp")
                .unwrap();
            let config = dir.path().join("bus.conf");
            std::fs::write(
                &config,
                format!(
                    r#"<busconfig>
  <type>session</type>
  <listen>unix:dir={}</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>"#,
                    dir.path().display()
                ),
            )
            .unwrap();
            let mut daemon = Command::new("dbus-daemon")
                .arg(format!("--config-file={}", config.display()))
                .args(["--nofork", "--print-address=1"])
                .stdout(Stdio::piped())
                .spawn()
                .expect("the tests need dbus-daemon on PATH");
            let mut address = String::new();
            BufReader::new(daemon.stdout.take().unwrap())
                .read_line(&mut address)
                .unwrap();
            Self {
                daemon,
                address: address.trim().to_owned(),
                _dir: dir,
            }
        }

        async fn connect(&self) -> zbus::Connection {
            zbus::connection::Builder::address(self.address.as_str())
                .unwrap()
                .build()
                .await
                .unwrap()
        }
    }

    impl Drop for PrivateBus {
        fn drop(&mut self) {
            _ = self.daemon.kill();
            _ = self.daemon.wait();
        }
    }

    #[derive(Default)]
    struct Device {
        level: i32,
        max: i32,
        writes: Vec<i32>,
    }

    /// UPower's composite keyboard backlight, as far as the daemon uses it.
    struct MockUPower(Arc<Mutex<Device>>);

    #[zbus::interface(name = "org.freedesktop.UPower.KbdBacklight")]
    impl MockUPower {
        fn get_brightness(&self) -> i32 {
            self.0.lock().unwrap().level
        }

        fn get_max_brightness(&self) -> i32 {
            self.0.lock().unwrap().max
        }

        async fn set_brightness(
            &self,
            value: i32,
            #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        ) -> zbus::fdo::Result<()> {
            let level = {
                let mut device = self.0.lock().unwrap();
                device.writes.push(value);
                device.level = value.clamp(0, device.max);
                device.level
            };
            _ = Self::brightness_changed(&emitter, level).await;
            Ok(())
        }

        #[zbus(signal)]
        async fn brightness_changed(emitter: &SignalEmitter<'_>, value: i32) -> zbus::Result<()>;
    }

    struct Rig {
        device: Arc<Mutex<Device>>,
        upower: zbus::Connection,
        backlight: KbdBacklight,
        _bus: PrivateBus,
    }

    impl Rig {
        async fn new(level: i32, max: i32) -> Self {
            let bus = PrivateBus::start();
            let device = Arc::new(Mutex::new(Device {
                level,
                max,
                writes: Vec::new(),
            }));
            let upower = bus.connect().await;
            upower
                .object_server()
                .at(
                    "/org/freedesktop/UPower/KbdBacklight",
                    MockUPower(device.clone()),
                )
                .await
                .unwrap();
            upower.request_name("org.freedesktop.UPower").await.unwrap();
            let backlight = KbdBacklight::new(&bus.connect().await).await.unwrap();
            Self {
                device,
                upower,
                backlight,
                _bus: bus,
            }
        }

        fn level(&self) -> i32 {
            self.device.lock().unwrap().level
        }

        fn writes(&self) -> Vec<i32> {
            self.device.lock().unwrap().writes.clone()
        }

        /// A change made outside the daemon, such as a firmware hotkey.
        async fn change_elsewhere(&self, level: i32) {
            self.device.lock().unwrap().level = level;
            let emitter =
                SignalEmitter::new(&self.upower, "/org/freedesktop/UPower/KbdBacklight").unwrap();
            MockUPower::brightness_changed(&emitter, level)
                .await
                .unwrap();
        }
    }

    async fn next_change(changes: &mut (impl Stream<Item = i32> + Unpin)) -> i32 {
        tokio::time::timeout(Duration::from_secs(5), changes.next())
            .await
            .expect("no BrightnessChanged within 5 s")
            .unwrap()
    }

    #[tokio::test]
    async fn the_up_and_down_keys_move_one_level() {
        let rig = Rig::new(1, 3).await;

        assert_eq!(rig.backlight.step(1).await, Some(2));
        assert_eq!(rig.level(), 2);
        assert_eq!(rig.backlight.step(-1).await, Some(1));
        assert_eq!(rig.backlight.step(-1).await, Some(0));
        assert_eq!(rig.level(), 0);
        assert_eq!(rig.writes(), [2, 1, 0]);
    }

    #[tokio::test]
    async fn the_level_stops_at_off_and_at_full() {
        let rig = Rig::new(3, 3).await;
        assert_eq!(rig.backlight.step(1).await, Some(3));

        rig.device.lock().unwrap().level = 0;
        assert_eq!(rig.backlight.step(-1).await, Some(0));

        assert_eq!(rig.backlight.set_brightness(9).await, Some(3));
        assert_eq!(rig.backlight.set_brightness(-4).await, Some(0));
        // Pressing past either end writes nothing, so no OSD shows a change.
        assert_eq!(rig.writes(), [3, 0]);
    }

    #[tokio::test]
    async fn the_toggle_turns_the_light_off_and_back_to_its_level() {
        let rig = Rig::new(2, 3).await;

        assert_eq!(rig.backlight.toggle().await, Some(0));
        assert_eq!(rig.level(), 0);
        assert_eq!(rig.backlight.toggle().await, Some(2));
        assert_eq!(rig.level(), 2);
    }

    #[tokio::test]
    async fn the_toggle_lights_an_unknown_level_at_full() {
        let rig = Rig::new(0, 3).await;

        assert_eq!(rig.backlight.toggle().await, Some(3));
    }

    #[tokio::test]
    async fn without_a_keyboard_backlight_nothing_is_written() {
        // UPower's composite device reports a maximum of 0 when there is none.
        let rig = Rig::new(0, 0).await;

        assert_eq!(rig.backlight.brightness().await, None);
        assert_eq!(rig.backlight.step(1).await, None);
        assert_eq!(rig.backlight.step(-1).await, None);
        assert_eq!(rig.backlight.toggle().await, None);
        assert_eq!(rig.backlight.set_brightness(1).await, None);
        assert!(rig.writes().is_empty());
    }

    #[tokio::test]
    async fn without_upower_nothing_fails() {
        let bus = PrivateBus::start();
        let backlight = KbdBacklight::new(&bus.connect().await).await.unwrap();

        assert_eq!(backlight.brightness().await, None);
        assert_eq!(backlight.step(1).await, None);
        assert_eq!(backlight.toggle().await, None);
    }

    #[tokio::test]
    async fn changes_made_elsewhere_are_followed() {
        let rig = Rig::new(0, 3).await;
        let mut changes = Box::pin(rig.backlight.changes().await.unwrap());

        rig.backlight.step(1).await;
        assert_eq!(next_change(&mut changes).await, 1);

        rig.change_elsewhere(3).await;
        let level = next_change(&mut changes).await;
        assert_eq!(level, 3);
        rig.backlight.observe(level);
        // The toggle restores the level set elsewhere, not its own last one.
        assert_eq!(rig.backlight.toggle().await, Some(0));
        assert_eq!(rig.backlight.toggle().await, Some(3));
    }

    /// The keys reach the daemon only through the shipped system actions.
    #[test]
    fn the_shipped_keyboard_actions_call_the_daemon() {
        let shipped = include_str!("../data/system_actions.ron");
        for (action, method) in [
            ("KeyboardBrightnessUp", "IncreaseKeyboardBrightness"),
            ("KeyboardBrightnessDown", "DecreaseKeyboardBrightness"),
            ("KeyboardBrightnessToggle", "ToggleKeyboardBacklight"),
        ] {
            let line = shipped
                .lines()
                .find(|line| line.trim_start().starts_with(&format!("{action}:")))
                .unwrap_or_else(|| panic!("{action} is not shipped"));
            assert!(
                line.contains(&format!(
                    "{DAEMON} {DAEMON_PATH} {DAEMON} {method}\"",
                    DAEMON = crate::DBUS_NAME,
                    DAEMON_PATH = crate::DBUS_PATH
                )),
                "{line}"
            );
        }
    }

    #[test]
    fn many_levels_take_at_most_ten_presses() {
        let mut level = 0;
        let mut presses = 0;
        while level < 255 {
            level = step_target(level, 255, 1);
            presses += 1;
        }
        assert_eq!(presses, 10);
        assert_eq!(step_target(255, 255, -1), 230);
        assert_eq!(step_target(0, 2, 1), 1);
        assert_eq!(step_target(2, 2, -1), 1);
    }
}
