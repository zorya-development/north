use leptos::prelude::*;
use north_dto::Project;

#[component]
pub fn ProjectPrefix(project_id: Option<i64>, projects: Signal<Vec<Project>>) -> impl IntoView {
    move || {
        if let Some(pid) = project_id {
            let projects = projects.get();
            projects.iter().find(|p| p.id == pid).map(|project| {
                let color = project.color.clone();
                let title = project.title.clone();
                let href = format!("/projects/{pid}");
                view! {
                    <span
                        class="text-sm font-medium mr-1"
                        style=format!("color: {color}")
                    >
                        "@"
                        <a
                            href=href
                            class="hover:underline"
                            on:click=move |ev: leptos::ev::MouseEvent| {
                                ev.stop_propagation();
                            }
                        >
                            {title}
                        </a>
                    </span>
                }
                .into_any()
            })
        } else {
            Some(
                view! {
                    <span class="text-sm font-medium mr-1 text-text-tertiary">
                        "@"
                        <a
                            href="/inbox"
                            class="hover:underline"
                            on:click=move |ev: leptos::ev::MouseEvent| {
                                ev.stop_propagation();
                            }
                        >
                            "Inbox"
                        </a>
                    </span>
                }
                .into_any(),
            )
        }
    }
}
