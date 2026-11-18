// SPDX-License-Identifier: MPL-2.0
#![allow(deprecated)] // Derives on deprecated variants produce warnings...

use serde::{Deserialize, Serialize};

/// An operation which may be bound to a keyboard shortcut.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Action {
    /// Route an app convention, such as Copy or New tab, to the focused app
    App(AppAction),

    /// Capture every display to a file, one per display
    CaptureScreen,

    /// Capture every display to the clipboard
    CaptureScreenClipboard,

    /// Drag out an area, or pick a window, and capture it to a file
    CaptureSelection,

    /// Drag out an area, or pick a window, and capture it to the clipboard
    CaptureSelectionClipboard,

    /// Show the capture toolbar for screenshots and screen recordings
    CaptureToolbar,

    /// Close the active window
    Close,

    /// Show a debug overlay, if enabled in the compositor build
    Debug,

    /// Disable a default shortcut binding
    Disable,

    /// Change focus to the window or workspace in the given direction
    Focus(FocusDirection),

    /// Move keyboard focus into the active window's Halo
    FocusHalo,

    /// Change focus to the last workspace
    LastWorkspace,

    /// Maximize the active window
    Maximize,

    /// Sets the active window to fullscreen
    Fullscreen,

    #[deprecated]
    /// Migrate the active workspace to the next output
    MigrateWorkspaceToNextOutput,

    /// Migrate the active workspace to the output in the given direction
    MigrateWorkspaceToOutput(Direction),

    #[deprecated]
    /// Migrate the active workspace to the previous output
    MigrateWorkspaceToPreviousOutput,

    /// Minimize the active window
    Minimize,

    /// Move a window in the given direction
    Move(Direction),

    /// Choose a desktop to move the active window to, and follow it there
    MoveToDesktop,

    /// Move a window to the last workspace
    MoveToLastWorkspace,

    #[deprecated]
    /// Move a window to the next output
    MoveToNextOutput,

    /// Move a window to the next workspace
    MoveToNextWorkspace,

    /// Move a window to the given output
    MoveToOutput(Direction),

    #[deprecated]
    /// Move a window to the previous output
    MoveToPreviousOutput,

    /// Move a window to the previous workspace
    MoveToPreviousWorkspace,

    /// Move a window to the given workspace
    MoveToWorkspace(u8),

    #[deprecated]
    /// Change focus to the next output
    NextOutput,

    /// Change to the next workspace — the vertical axis, above desktops
    NextRealm,

    /// Change focus to the next workspace
    NextWorkspace,

    /// Change the orientation of a tiling group
    Orientation(Orientation),

    /// Pick a colour from the screen and copy its value
    PickColor,

    #[deprecated]
    /// Change focus to the previous output
    PreviousOutput,

    /// Change to the previous workspace — the vertical axis, above desktops
    PreviousRealm,

    /// Change focus to the previous workspace
    PreviousWorkspace,

    /// Resize the active window in a given direction
    Resizing(ResizeDirection),

    /// Move a window to the last workspace
    SendToLastWorkspace,

    #[deprecated]
    /// Move a window to the next output
    SendToNextOutput,

    /// Move a window to the next workspace
    SendToNextWorkspace,

    /// Move a window to the output in the given direction
    SendToOutput(Direction),

    #[deprecated]
    /// Move a window to the previous output
    SendToPreviousOutput,

    /// Move a window to the previous workspace
    SendToPreviousWorkspace,

    /// Move a window to the given workspace
    SendToWorkspace(u8),

    /// Swap positions of the active window with another
    SwapWindow,

    /// Move to an output in the given direction
    SwitchOutput(Direction),

    /// Perform a common system operation
    System(System),

    /// Execute a command with any given arguments
    Spawn(String),

    /// Hold to dictate into the focused text field; releasing finishes
    SpeechToText,

    /// Stop the running screen recording
    StopRecording,

    /// Stop the compositor
    Terminate,

    /// Toggle the orientation of a tiling group
    ToggleOrientation,

    /// Toggle window stacking for the active window
    ToggleStacking,

    /// Toggle the sticky state of the active window
    ToggleSticky,

    /// Toggle tiling mode of the active workspace
    ToggleTiling,

    /// Toggle between tiling and floating window states for the active window
    ToggleWindowFloating,

    /// Open or close the active window's command palette
    WindowCommands,

    /// Change focus to the given workspace ID
    Workspace(u8),

    /// Enter Magnification / Increase the zoom level by the configured interval
    ZoomIn,

    /// Leave Magnification / Decrease the zoom level by the configured interval
    ZoomOut,
}

/// Common system operations which may be controlled by system commands
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum System {
    /// Opens the application library
    AppLibrary,
    /// Decreases screen brightness
    BrightnessDown,
    /// Increases screen brightness
    BrightnessUp,
    /// Opens the calculator
    Calculator,
    /// Opens Chord (the name is kept for the configs that already bind it)
    ChatPanel,
    /// Toggles display mode
    DisplayToggle,
    /// Opens the home folder in a system default file browser
    HomeFolder,
    /// Switch the currently-active input source
    InputSourceSwitch,
    /// Decreases keyboard brightness
    KeyboardBrightnessDown,
    /// Increases keyboard brightness
    KeyboardBrightnessUp,
    /// Turns the keyboard backlight off, or back on at its last level
    KeyboardBrightnessToggle,
    /// Opens the launcher
    Launcher,
    /// Locks the screen
    LockScreen,
    /// Logs out
    LogOut,
    /// Mutes the active audio output
    Mute,
    /// Mutes the active microphone
    MuteMic,
    /// Plays and Pauses audio
    PlayPause,
    /// Goes to the next track
    PlayNext,
    /// Goes to the previous track
    PlayPrev,
    /// Power off button handler
    PowerOff,
    /// Toggles the screen reader
    ScreenReader,
    /// Takes a screenshot
    Screenshot,
    /// Opens the system settings application
    Settings,
    /// Opens the Spotlight search overlay
    Spotlight,
    /// Opens the AgentOS start menu
    StartMenu,
    /// Suspend the system
    Suspend,
    /// Opens the system default terminal
    Terminal,
    /// Toggles touchpad on/off
    TouchpadToggle,
    /// Lowers the volume of the active audio output
    VolumeLower,
    /// Raises the volume of the active audio output
    VolumeRaise,
    /// Opens the system default web browser
    WebBrowser,
    /// Opens the (alt+tab) window switcher
    WindowSwitcher,
    /// Opens the (alt+shift+tab) window switcher
    WindowSwitcherPrevious,
    /// Opens the window switcher over the focused app's windows only
    WindowSwitcherApp,
    /// Opens the same-app window switcher, cycling backwards
    WindowSwitcherAppPrevious,
    /// Opens the workspace overview
    WorkspaceOverview,
}

/// A command that belongs to the focused app. The compositor routes the chord;
/// the app decides what it means.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum AppAction {
    /// Open a new window of the app
    NewWindow,
    /// Open a new tab in the app
    NewTab,
    /// Cut the selection
    Cut,
    /// Copy the selection
    Copy,
    /// Paste the clipboard
    Paste,
    /// Undo the last edit
    Undo,
    /// Redo the last undone edit
    Redo,
    /// Select everything
    SelectAll,
}

/// Defines the direction of an operation
#[derive(Copy, Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl std::ops::Not for Direction {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

/// Defines the direction to focus towards
#[derive(Copy, Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum FocusDirection {
    Left,
    Right,
    Up,
    Down,
    In,
    Out,
}

/// Defines the direction to resize towards
#[derive(Copy, Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ResizeDirection {
    Inwards,
    Outwards,
}

/// Defines the edge of a window to resize from
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ResizeEdge {
    Bottom,
    BottomLeft,
    BottomRight,
    Left,
    Right,
    Top,
    TopLeft,
    TopRight,
}

/// Tiling orientation for a tiling window group
#[derive(Copy, Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl std::ops::Not for Orientation {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Action;

    #[test]
    fn window_commands_round_trips_through_ron() {
        let action: Action = ron::from_str("WindowCommands").unwrap();
        assert_eq!(action, Action::WindowCommands);
        assert_eq!(ron::to_string(&action).unwrap(), "WindowCommands");
    }

    #[test]
    fn speech_to_text_round_trips_through_ron() {
        let action: Action = ron::from_str("SpeechToText").unwrap();
        assert_eq!(action, Action::SpeechToText);
        assert_eq!(ron::to_string(&action).unwrap(), "SpeechToText");
    }

    #[test]
    fn capture_actions_round_trip_through_ron() {
        for (text, action) in [
            ("CaptureScreen", Action::CaptureScreen),
            ("CaptureScreenClipboard", Action::CaptureScreenClipboard),
            ("CaptureSelection", Action::CaptureSelection),
            (
                "CaptureSelectionClipboard",
                Action::CaptureSelectionClipboard,
            ),
            ("CaptureToolbar", Action::CaptureToolbar),
            ("StopRecording", Action::StopRecording),
            ("PickColor", Action::PickColor),
        ] {
            assert_eq!(ron::from_str::<Action>(text).unwrap(), action);
            assert_eq!(ron::to_string(&action).unwrap(), text);
        }
    }

    #[test]
    fn halo_actions_round_trip_through_ron() {
        for (text, action) in [
            ("FocusHalo", Action::FocusHalo),
            ("MoveToDesktop", Action::MoveToDesktop),
        ] {
            assert_eq!(ron::from_str::<Action>(text).unwrap(), action);
            assert_eq!(ron::to_string(&action).unwrap(), text);
        }
    }

    #[test]
    fn shortcut_contract_actions_round_trip_through_ron() {
        use super::{AppAction, System};
        for (text, action) in [
            ("App(NewWindow)", Action::App(AppAction::NewWindow)),
            ("App(NewTab)", Action::App(AppAction::NewTab)),
            ("App(Cut)", Action::App(AppAction::Cut)),
            ("App(Copy)", Action::App(AppAction::Copy)),
            ("App(Paste)", Action::App(AppAction::Paste)),
            ("App(Undo)", Action::App(AppAction::Undo)),
            ("App(Redo)", Action::App(AppAction::Redo)),
            ("App(SelectAll)", Action::App(AppAction::SelectAll)),
            (
                "System(WindowSwitcherApp)",
                Action::System(System::WindowSwitcherApp),
            ),
            (
                "System(WindowSwitcherAppPrevious)",
                Action::System(System::WindowSwitcherAppPrevious),
            ),
        ] {
            assert_eq!(ron::from_str::<Action>(text).unwrap(), action);
            assert_eq!(ron::to_string(&action).unwrap(), text);
        }
    }
}
