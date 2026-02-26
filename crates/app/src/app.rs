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
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <script>
                    "(function(){try{var t=localStorage.getItem('north-theme');\
                    if(t==='dark'||(t!=='light'&&window.matchMedia\
                    ('(prefers-color-scheme:dark)').matches))\
                    {document.documentElement.classList.add('dark')}\
                    }catch(e){}})()"
                </script>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous"/>
                <link
                    href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap"
                    rel="stylesheet"
                />
                <link rel="icon" href="/public/favicon.ico"/>
                <link rel="stylesheet" id="leptos" href="/pkg/north.css"/>
            </head>
            <body class="bg-bg-primary text-text-primary font-sans antialiased">
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
                    path=path!("/someday")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::someday::SomedayPage/>
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
                <Route
                    path=path!("/filters/new")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::filter::FilterPage/>
                            </components::layout::AppLayout>
                        }
                    }
                />
                <Route
                    path=path!("/filters/help")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::filter_help::FilterHelpPage/>
                            </components::layout::AppLayout>
                        }
                    }
                />
                <Route
                    path=path!("/filters/:id")
                    view=|| {
                        view! {
                            <components::layout::AppLayout>
                                <pages::filter::FilterPage/>
                            </components::layout::AppLayout>
                        }
                    }
                />
            </Routes>
        </Router>
    }
}
