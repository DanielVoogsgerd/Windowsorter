//! Minimal, more-abstract rule engine for moving windows between workspaces.
use std::collections::HashSet;

/// A simple representation of a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// Window class (e.g., "firefox", "Alacritty"). Lowercase preferred.
    pub class: String,
    /// Numeric workspace index (1-based).
    pub workspace: u32,
}

/// What action to take for a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Do nothing.
    None,
    /// Move to the given workspace.
    MoveTo(u32),
}

/// A general application type: matches a set of classes, provides a default
/// workspace, a set of forbidden workspaces and an optional mandatory workspace
/// where instances must live.
#[derive(Debug, Clone)]
pub struct AppType {
    /// Human readable name
    pub name: String,
    /// Lowercased class names to match
    pub classes: HashSet<String>,
    /// Default workspace to move to when remapping from a forbidden location.
    pub default_workspace: u32,
    /// Workspaces that this app must not be opened on (will be remapped)
    pub forbidden: HashSet<u32>,
    /// If set, this is the mandatory workspace for the app type. If present,
    /// windows of this type should always be moved here.
    pub mandatory_workspace: Option<u32>,
}

impl AppType {
    /// Convenience constructor from slices
    pub fn new<S: Into<String>>(name: S, classes: &[&str], default_workspace: u32, forbidden: &[u32], mandatory_workspace: Option<u32>) -> Self {
        AppType {
            name: name.into(),
            classes: classes.iter().map(|s| s.to_string()).collect(),
            default_workspace,
            forbidden: forbidden.iter().copied().collect(),
            mandatory_workspace,
        }
    }

    /// Returns true if the given lowercased class matches this app type.
    pub fn matches_class(&self, class_lc: &str) -> bool {
        self.classes.contains(class_lc)
    }
}

/// Rules configuration: a list of `AppType`s. The first matching app type wins.
#[derive(Debug, Clone)]
pub struct Rules {
    pub app_types: Vec<AppType>,
}

/// Decide what to do with a newly opened or focused window.
pub fn decide_action(window: &Window, rules: &Rules) -> Action {
    let class = window.class.to_lowercase();

    for app in &rules.app_types {
        if app.matches_class(&class) {
            if let Some(mand_ws) = app.mandatory_workspace {
                if window.workspace != mand_ws {
                    return Action::MoveTo(mand_ws);
                }
                return Action::None;
            }

            if app.forbidden.contains(&window.workspace) {
                return Action::MoveTo(app.default_workspace);
            }

            return Action::None;
        }
    }

    Action::None
}
