//! Empty States + Personality Moments (Part 44 of spec).
//!
//! Locked strings -- never localized, never changed, always lowercase.
//! These define the personality of vitusOS: clean, understated, human.

/// All personality strings in vitusOS (Part 44).
/// These are the EXACT strings. Never modify, never translate, never capitalize.
pub mod strings {
    // -- Empty States --
    pub const FILER_EMPTY_FOLDER: &str = "nothing here yet";
    pub const FILER_NO_SEARCH_RESULTS: &str = "no files match";
    pub const PATHFINDER_NO_RESULTS: &str = "nothing found for";
    pub const COCKPITVIEW_ONE_WINDOW: &str = "open more apps to fill the space";
    pub const PATHFINDER_FIRST_OPEN: &str = "what are you looking for?";
    pub const PATHFINDER_PLACEHOLDER: &str = "Search vitusOS";
    pub const FILER_FIRST_OPEN: &str = "this is your home";

    // -- Shutdown / Restart (LOCKED, Part 29) --
    pub const SHUTDOWN: &str = "goodbye";
    pub const RESTART: &str = "i'll see you in a bit";

    // -- First Boot Welcome (Part 37) --
    pub const WELCOME_STEP3_TITLE: &str = "you're all set";
    pub const WELCOME_STEP3_SUBTITLE: &str = "welcome to vitusOS";
    pub const WELCOME_STEP3_BUTTON: &str = "let's go";

    // -- Lock Screen --
    // Time display: 48px Inter Light
    // Password field appears on first keypress -- no placeholder text
    // No "Enter Password" text -- the field IS the instruction

    // -- Install Complete --
    // No modal -- icon travels to Dock, one bounce, no text
}

/// Counter for CockpitView one-window message (Part 44).
/// Shows "open more apps to fill the space" only the first 3 times.
pub struct PersonalityCounter {
    cockpit_one_window_shown: u32,
}

impl PersonalityCounter {
    pub const COCKPIT_ONE_WINDOW_MAX: u32 = 3;

    pub fn new() -> Self {
        Self { cockpit_one_window_shown: 0 }
    }

    /// Returns true if the CockpitView one-window message should be shown.
    pub fn should_show_cockpit_one_window(&mut self) -> bool {
        if self.cockpit_one_window_shown < Self::COCKPIT_ONE_WINDOW_MAX {
            self.cockpit_one_window_shown += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_strings_exact() {
        // These strings are LOCKED -- never change
        assert_eq!(strings::SHUTDOWN, "goodbye");
        assert_eq!(strings::RESTART, "i'll see you in a bit");
        assert_eq!(strings::WELCOME_STEP3_TITLE, "you're all set");
        assert_eq!(strings::WELCOME_STEP3_BUTTON, "let's go");
        assert_eq!(strings::FILER_EMPTY_FOLDER, "nothing here yet");
        assert_eq!(strings::PATHFINDER_FIRST_OPEN, "what are you looking for?");
    }

    #[test]
    fn test_cockpitview_one_window_counter() {
        let mut counter = PersonalityCounter::new();

        assert!(counter.should_show_cockpit_one_window()); // 1
        assert!(counter.should_show_cockpit_one_window()); // 2
        assert!(counter.should_show_cockpit_one_window()); // 3
        assert!(!counter.should_show_cockpit_one_window()); // 4 -- stopped
        assert!(!counter.should_show_cockpit_one_window()); // still stopped
    }
}
