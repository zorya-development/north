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
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <link rel="stylesheet" id="leptos" href="/pkg/north.css"/>
            </head>
            <body class="bg-white text-teal-950 font-sans">
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
            <Routes fallback=|| view! { <p class="p-8 text-sage-400">"Page not found"</p> }>
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
            </Routes>
        </Router>
    }
}
