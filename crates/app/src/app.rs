use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::{
    components::{Redirect, Route, Router, Routes},
    path,
};

use crate::components;
use crate::pages;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" style="color-scheme: dark">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <link rel="stylesheet" id="leptos" href="/pkg/north.css"/>
            </head>
            <body class="bg-bg-primary text-text-primary font-sans">
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <Routes fallback=|| view! { <p class="p-8 text-text-secondary">"Page not found"</p> }>
                <Route path=path!("/login") view=pages::login::LoginPage/>
                <Route
                    path=path!("/")
                    view=|| view! { <Redirect path="/inbox"/> }
                />
                <Route
                    path=path!("/inbox")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::inbox::InboxPage/>
                            </components::layout::AppLayout>
                        }
                    }
                />
                <Route
                    path=path!("/today")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::today::TodayPage/>
                            </components::layout::AppLayout>
                        }
                    }
                />
                <Route
                    path=path!("/tasks")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::all_tasks::AllTasksPage/>
                            </components::layout::AppLayout>
                        }
                    }
                />
                <Route
                    path=path!("/projects/:id")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::project::ProjectPage/>
                            </components::layout::AppLayout>
                        }
                    }
                />
                <Route
                    path=path!("/archive")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::archive::ArchivePage/>
                            </components::layout::AppLayout>
                        }
                    }
                />
                <Route
                    path=path!("/review")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::review::ReviewPage/>
                            </components::layout::AppLayout>
                        }
                    }
                />
                <Route
                    path=path!("/settings")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::settings::SettingsPage/>
                            </components::layout::AppLayout>
                        }
                    }
                />
            </Routes>
        </Router>
    }
}
