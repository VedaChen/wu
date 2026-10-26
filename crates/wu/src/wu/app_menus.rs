use gpui::{App, Menu, MenuItem, OsAction};
use release_channel::ReleaseChannel;
use terminal_view::terminal_panel;
use wu_actions::{Quit, debug_panel, dev, git_panel, project_panel};

pub fn app_menus(cx: &mut App) -> Vec<Menu> {
    let mut view_items = vec![
        MenuItem::action(
            i18n::tr("Zoom In"),
            wu_actions::IncreaseBufferFontSize { persist: false },
        ),
        MenuItem::action(
            i18n::tr("Zoom Out"),
            wu_actions::DecreaseBufferFontSize { persist: false },
        ),
        MenuItem::action(
            i18n::tr("Reset Zoom"),
            wu_actions::ResetBufferFontSize { persist: false },
        ),
        MenuItem::action(
            i18n::tr("Reset All Zoom"),
            wu_actions::ResetAllZoom { persist: false },
        ),
        MenuItem::separator(),
        MenuItem::action(i18n::tr("Toggle Left Dock"), workspace::ToggleLeftDock),
        MenuItem::action(i18n::tr("Toggle Right Dock"), workspace::ToggleRightDock),
        MenuItem::action(i18n::tr("Toggle Bottom Dock"), workspace::ToggleBottomDock),
        MenuItem::action(i18n::tr("Toggle All Docks"), workspace::ToggleAllDocks),
        MenuItem::submenu(Menu {
            name: i18n::tr("Editor Layout"),
            disabled: false,
            items: vec![
                MenuItem::action(i18n::tr("Split Up"), workspace::SplitUp::default()),
                MenuItem::action(i18n::tr("Split Down"), workspace::SplitDown::default()),
                MenuItem::action(i18n::tr("Split Left"), workspace::SplitLeft::default()),
                MenuItem::action(i18n::tr("Split Right"), workspace::SplitRight::default()),
            ],
        }),
        MenuItem::separator(),
        MenuItem::action(i18n::tr("Project Panel"), project_panel::ToggleFocus),
        MenuItem::action(i18n::tr("Outline Panel"), outline_panel::ToggleFocus),
        MenuItem::action(i18n::tr("Terminal Panel"), terminal_panel::Toggle),
        MenuItem::action(i18n::tr("Debugger Panel"), debug_panel::ToggleFocus),
        MenuItem::action(i18n::tr("Git Panel"), git_panel::ToggleFocus),
        MenuItem::separator(),
        MenuItem::action(i18n::tr("Diagnostics"), diagnostics::Deploy),
        MenuItem::separator(),
    ];

    if ReleaseChannel::try_global(cx) == Some(ReleaseChannel::Dev) {
        view_items.push(MenuItem::action(
            i18n::tr("Toggle GPUI Inspector"),
            dev::ToggleInspector,
        ));
        view_items.push(MenuItem::separator());
    }

    vec![
        Menu {
            name: i18n::tr("Wu"),
            disabled: false,
            items: vec![
                MenuItem::action(i18n::tr("About Wu"), wu_actions::About),
                MenuItem::action(i18n::tr("Check for Updates"), auto_update::Check),
                MenuItem::separator(),
                MenuItem::submenu(Menu::new(i18n::tr("Settings")).items([
                    MenuItem::action(i18n::tr("Open Settings"), wu_actions::OpenSettings),
                    MenuItem::action(i18n::tr("Open Settings File"), super::OpenSettingsFile),
                    MenuItem::action(
                        i18n::tr("Open Project Settings"),
                        wu_actions::OpenProjectSettings,
                    ),
                    MenuItem::action(
                        i18n::tr("Open Project Settings File"),
                        super::OpenProjectSettingsFile,
                    ),
                    MenuItem::action(
                        i18n::tr("Open Default Settings"),
                        super::OpenDefaultSettings,
                    ),
                    MenuItem::separator(),
                    MenuItem::action(i18n::tr("Open Keymap"), wu_actions::OpenKeymap),
                    MenuItem::action(i18n::tr("Open Keymap File"), wu_actions::OpenKeymapFile),
                    MenuItem::action(
                        i18n::tr("Open Default Key Bindings"),
                        wu_actions::OpenDefaultKeymap,
                    ),
                    MenuItem::separator(),
                    MenuItem::action(
                        i18n::tr("Select Theme..."),
                        wu_actions::theme_selector::Toggle::default(),
                    ),
                    MenuItem::action(
                        i18n::tr("Select Icon Theme..."),
                        wu_actions::icon_theme_selector::Toggle::default(),
                    ),
                ])),
                MenuItem::separator(),
                #[cfg(target_os = "macos")]
                MenuItem::os_submenu(i18n::tr("Services"), gpui::SystemMenuType::Services),
                MenuItem::separator(),
                MenuItem::action(i18n::tr("Extensions"), wu_actions::Extensions::default()),
                #[cfg(not(target_os = "windows"))]
                MenuItem::action(i18n::tr("Install CLI"), install_cli::InstallCliBinary),
                MenuItem::separator(),
                #[cfg(target_os = "macos")]
                MenuItem::action(i18n::tr("Hide Wu"), super::Hide),
                #[cfg(target_os = "macos")]
                MenuItem::action(i18n::tr("Hide Others"), super::HideOthers),
                #[cfg(target_os = "macos")]
                MenuItem::action(i18n::tr("Show All"), super::ShowAll),
                MenuItem::separator(),
                MenuItem::action(i18n::tr("Quit Wu"), Quit),
            ],
        },
        Menu {
            name: i18n::tr("File"),
            disabled: false,
            items: vec![
                MenuItem::action(i18n::tr("New"), workspace::NewFile),
                MenuItem::action(i18n::tr("New Window"), workspace::NewWindow),
                MenuItem::separator(),
                #[cfg(not(target_os = "macos"))]
                MenuItem::action(i18n::tr("Open File..."), workspace::OpenFiles),
                MenuItem::action(
                    if cfg!(not(target_os = "macos")) {
                        i18n::tr("Open Folder...")
                    } else {
                        i18n::tr("Open…")
                    },
                    workspace::Open::default(),
                ),
                MenuItem::action(i18n::tr("Open Recent…"), wu_actions::OpenRecent::default()),
                MenuItem::action(i18n::tr("Open Remote…"), wu_actions::OpenRemote::default()),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Add Folder to Project…"),
                    workspace::AddFolderToProject,
                ),
                MenuItem::separator(),
                MenuItem::action(i18n::tr("Save"), workspace::Save { save_intent: None }),
                MenuItem::action(i18n::tr("Save As…"), workspace::SaveAs),
                MenuItem::action(
                    i18n::tr("Save All"),
                    workspace::SaveAll { save_intent: None },
                ),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Close Editor"),
                    workspace::CloseActiveItem {
                        save_intent: None,
                        close_pinned: true,
                    },
                ),
                MenuItem::action(i18n::tr("Close Project"), workspace::CloseProject),
                MenuItem::action(i18n::tr("Close Window"), workspace::CloseWindow),
            ],
        },
        Menu {
            name: i18n::tr("Edit"),
            disabled: false,
            items: vec![
                MenuItem::os_action(i18n::tr("Undo"), editor::actions::Undo, OsAction::Undo),
                MenuItem::os_action(i18n::tr("Redo"), editor::actions::Redo, OsAction::Redo),
                MenuItem::separator(),
                MenuItem::os_action(i18n::tr("Cut"), editor::actions::Cut, OsAction::Cut),
                MenuItem::os_action(i18n::tr("Copy"), editor::actions::Copy, OsAction::Copy),
                MenuItem::action(i18n::tr("Copy and Trim"), editor::actions::CopyAndTrim),
                MenuItem::os_action(i18n::tr("Paste"), editor::actions::Paste, OsAction::Paste),
                MenuItem::separator(),
                MenuItem::action(i18n::tr("Find"), search::buffer_search::Deploy::find()),
                MenuItem::action(
                    i18n::tr("Find in Project"),
                    workspace::DeploySearch::default(),
                ),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Toggle Line Comment"),
                    editor::actions::ToggleComments::default(),
                ),
            ],
        },
        Menu {
            name: i18n::tr("Selection"),
            disabled: false,
            items: vec![
                MenuItem::os_action(
                    i18n::tr("Select All"),
                    editor::actions::SelectAll,
                    OsAction::SelectAll,
                ),
                MenuItem::action(
                    i18n::tr("Expand Selection"),
                    editor::actions::SelectLargerSyntaxNode,
                ),
                MenuItem::action(
                    i18n::tr("Shrink Selection"),
                    editor::actions::SelectSmallerSyntaxNode,
                ),
                MenuItem::action(
                    i18n::tr("Select Next Sibling"),
                    editor::actions::SelectNextSyntaxNode,
                ),
                MenuItem::action(
                    i18n::tr("Select Previous Sibling"),
                    editor::actions::SelectPreviousSyntaxNode,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Add Cursor Above"),
                    editor::actions::AddSelectionAbove {
                        skip_soft_wrap: true,
                    },
                ),
                MenuItem::action(
                    i18n::tr("Add Cursor Below"),
                    editor::actions::AddSelectionBelow {
                        skip_soft_wrap: true,
                    },
                ),
                MenuItem::action(
                    i18n::tr("Select Next Occurrence"),
                    editor::actions::SelectNext {
                        replace_newest: false,
                    },
                ),
                MenuItem::action(
                    i18n::tr("Select Previous Occurrence"),
                    editor::actions::SelectPrevious {
                        replace_newest: false,
                    },
                ),
                MenuItem::action(
                    i18n::tr("Select All Occurrences"),
                    editor::actions::SelectAllMatches,
                ),
                MenuItem::separator(),
                MenuItem::action(i18n::tr("Move Line Up"), editor::actions::MoveLineUp),
                MenuItem::action(i18n::tr("Move Line Down"), editor::actions::MoveLineDown),
                MenuItem::action(
                    i18n::tr("Duplicate Selection"),
                    editor::actions::DuplicateLineDown,
                ),
            ],
        },
        Menu {
            name: i18n::tr("View"),
            disabled: false,
            items: view_items,
        },
        Menu {
            name: i18n::tr("Go"),
            disabled: false,
            items: vec![
                MenuItem::action(i18n::tr("Back"), workspace::GoBack),
                MenuItem::action(i18n::tr("Forward"), workspace::GoForward),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Command Palette..."),
                    wu_actions::command_palette::Toggle,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Go to File..."),
                    workspace::ToggleFileFinder::default(),
                ),
                // MenuItem::action(i18n::tr("Go to Symbol in Project"), project_symbols::Toggle),
                MenuItem::action(
                    i18n::tr("Go to Symbol in Editor..."),
                    wu_actions::outline::ToggleOutline,
                ),
                MenuItem::action(
                    i18n::tr("Go to Line/Column..."),
                    editor::actions::ToggleGoToLine,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Go to Definition"),
                    editor::actions::GoToDefinition::default(),
                ),
                MenuItem::action(
                    i18n::tr("Go to Declaration"),
                    editor::actions::GoToDeclaration::default(),
                ),
                MenuItem::action(
                    i18n::tr("Go to Type Definition"),
                    editor::actions::GoToTypeDefinition::default(),
                ),
                MenuItem::action(
                    i18n::tr("Find All References"),
                    editor::actions::FindAllReferences::default(),
                ),
                MenuItem::action(
                    i18n::tr("Show Incoming Calls"),
                    call_hierarchy::ShowIncomingCalls,
                ),
                MenuItem::action(
                    i18n::tr("Show Outgoing Calls"),
                    call_hierarchy::ShowOutgoingCalls,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Next Problem"),
                    editor::actions::GoToDiagnostic::default(),
                ),
                MenuItem::action(
                    i18n::tr("Previous Problem"),
                    editor::actions::GoToPreviousDiagnostic::default(),
                ),
            ],
        },
        Menu {
            name: i18n::tr("Run"),
            disabled: false,
            items: vec![
                MenuItem::action(
                    i18n::tr("Spawn Task"),
                    wu_actions::Spawn::ViaModal {
                        reveal_target: None,
                    },
                ),
                MenuItem::action(i18n::tr("Start Debugger"), debugger_ui::Start),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Edit tasks.json…"),
                    wu_actions::OpenProjectTasks,
                ),
                MenuItem::action(
                    i18n::tr("Edit debug.json…"),
                    wu_actions::OpenProjectDebugTasks,
                ),
                MenuItem::separator(),
                MenuItem::action(i18n::tr("Continue"), debugger_ui::Continue),
                MenuItem::action(i18n::tr("Step Over"), debugger_ui::StepOver),
                MenuItem::action(i18n::tr("Step Into"), debugger_ui::StepInto),
                MenuItem::action(i18n::tr("Step Out"), debugger_ui::StepOut),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Toggle Breakpoint"),
                    editor::actions::ToggleBreakpoint,
                ),
                MenuItem::action(
                    i18n::tr("Edit Breakpoint"),
                    editor::actions::EditLogBreakpoint,
                ),
                MenuItem::action(
                    i18n::tr("Clear All Breakpoints"),
                    debugger_ui::ClearAllBreakpoints,
                ),
            ],
        },
        Menu {
            name: i18n::tr("Window"),
            disabled: false,
            items: vec![
                MenuItem::action(i18n::tr("Minimize"), super::Minimize),
                MenuItem::action(i18n::tr("Zoom"), super::Zoom),
                MenuItem::separator(),
            ],
        },
        Menu {
            name: i18n::tr("Help"),
            disabled: false,
            items: vec![
                MenuItem::action(
                    i18n::tr("View Release Notes Locally"),
                    auto_update_ui::ViewReleaseNotesLocally,
                ),
                MenuItem::action(
                    i18n::tr("View Dependency Licenses"),
                    wu_actions::OpenLicenses,
                ),
                MenuItem::action(i18n::tr("Show Welcome"), onboarding::ShowWelcome),
                MenuItem::separator(),
                MenuItem::action(
                    i18n::tr("Documentation"),
                    super::OpenBrowser {
                        url: "https://wu.dstrml.org/docs".into(),
                    },
                ),
                MenuItem::action(
                    i18n::tr("Wu Repository"),
                    super::OpenBrowser {
                        url: "https://github.com/dstrml/wu".into(),
                    },
                ),
            ],
        },
    ]
}
