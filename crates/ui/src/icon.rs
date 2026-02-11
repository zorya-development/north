use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    Inbox,
    Today,
    Tasks,
    Review,
    Filter,
    Stats,
    Settings,
    Check,
    KebabMenu,
    Calendar,
    Folder,
    Plus,
    Archive,
    Tag,
    Edit,
    Save,
    QuestionMark,
    ChevronLeft,
    ChevronRight,
    Close,
    Subtask,
    ChevronDown,
    ChevronUp,
    Trash,
}

#[component]
pub fn Icon(kind: IconKind, #[prop(default = "w-4 h-4")] class: &'static str) -> impl IntoView {
    match kind {
        IconKind::Inbox => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/>
                <path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 \
                         2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 \
                         0 0 0-1.79 1.11z"/>
            </svg>
        }
        .into_any(),
        IconKind::Today => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
            </svg>
        }
        .into_any(),
        IconKind::Tasks => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 2 2 7l10 5 10-5-10-5z"/>
                <path d="m2 17 10 5 10-5"/>
                <path d="m2 12 10 5 10-5"/>
            </svg>
        }
        .into_any(),
        IconKind::Review => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="1 4 1 10 7 10"/>
                <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
            </svg>
        }
        .into_any(),
        IconKind::Filter => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
            </svg>
        }
        .into_any(),
        IconKind::Stats => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="20" x2="18" y2="10"/>
                <line x1="12" y1="20" x2="12" y2="4"/>
                <line x1="6" y1="20" x2="6" y2="14"/>
            </svg>
        }
        .into_any(),
        IconKind::Settings => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"/>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 \
                         0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 \
                         1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 \
                         2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 \
                         9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 \
                         1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 \
                         0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 \
                         1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 \
                         9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 \
                         0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 \
                         0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 \
                         1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 \
                         1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 \
                         1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 \
                         0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 \
                         0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 \
                         0-1.51 1z"/>
            </svg>
        }
        .into_any(),
        IconKind::Check => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="3"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12"/>
            </svg>
        }
        .into_any(),
        IconKind::KebabMenu => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="currentColor">
                <circle cx="12" cy="5" r="2"/>
                <circle cx="12" cy="12" r="2"/>
                <circle cx="12" cy="19" r="2"/>
            </svg>
        }
        .into_any(),
        IconKind::Calendar => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
                <line x1="16" y1="2" x2="16" y2="6"/>
                <line x1="8" y1="2" x2="8" y2="6"/>
                <line x1="3" y1="10" x2="21" y2="10"/>
            </svg>
        }
        .into_any(),
        IconKind::Folder => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 \
                         1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
        }
        .into_any(),
        IconKind::Plus => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"/>
                <line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
        }
        .into_any(),
        IconKind::Archive => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="21 8 21 21 3 21 3 8"/>
                <rect x="1" y="3" width="22" height="5"/>
                <line x1="10" y1="12" x2="14" y2="12"/>
            </svg>
        }
        .into_any(),
        IconKind::Tag => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 \
                         8.59a2 2 0 0 1 0 2.82z"/>
                <line x1="7" y1="7" x2="7.01" y2="7"/>
            </svg>
        }
        .into_any(),
        IconKind::Edit => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 \
                         0 0 0 2-2v-7"/>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 \
                         9.5-9.5z"/>
            </svg>
        }
        .into_any(),
        IconKind::Save => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 \
                         2 0 0 1-2 2z"/>
                <polyline points="17 21 17 13 7 13 7 21"/>
                <polyline points="7 3 7 8 15 8"/>
            </svg>
        }
        .into_any(),
        IconKind::QuestionMark => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"/>
                <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/>
                <line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
        }
        .into_any(),
        IconKind::ChevronLeft => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="15 18 9 12 15 6"/>
            </svg>
        }
        .into_any(),
        IconKind::ChevronRight => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="9 18 15 12 9 6"/>
            </svg>
        }
        .into_any(),
        IconKind::Close => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
        }
        .into_any(),
        IconKind::Subtask => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <path d="M6 3v12"/>
                <path d="M6 15a3 3 0 0 0 3 3h6"/>
                <polyline points="15 15 18 18 15 21"/>
            </svg>
        }
        .into_any(),
        IconKind::ChevronDown => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="6 9 12 15 18 9"/>
            </svg>
        }
        .into_any(),
        IconKind::ChevronUp => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="18 15 12 9 6 15"/>
            </svg>
        }
        .into_any(),
        IconKind::Trash => view! {
            <svg xmlns="http://www.w3.org/2000/svg" class=class viewBox="0 0 24 24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4\
                         a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            </svg>
        }
        .into_any(),
    }
}
