use leptos::prelude::*;
use north_ui::Modal;

#[component]
pub fn KeybindingsModal(open: ReadSignal<bool>, set_open: WriteSignal<bool>) -> impl IntoView {
    let keybindings: Vec<(&str, &str)> = vec![
        ("\u{2191} / \u{2193}", "Move cursor up/down"),
        ("\u{2190} / \u{2192}", "Go to parent / first child"),
        ("Shift+\u{2191}\u{2193}\u{2190}\u{2192}", "Reorder task"),
        ("Enter", "Edit task title"),
        ("Ctrl+Enter", "Create task after"),
        ("Shift+Enter", "Create task before"),
        ("Ctrl+Shift+Enter", "Create subtask"),
        ("Space", "Toggle complete"),
        ("E", "Open detail"),
        ("R", "Mark as reviewed"),
        ("Delete", "Delete task"),
        ("Escape", "Clear selection"),
        ("?", "This help"),
    ];

    view! {
        <Modal open=open set_open=set_open>
            <div data-testid="keybindings-modal" class="p-6">
                <h2 class="text-lg font-semibold text-text-primary mb-4">
                    "Keyboard shortcuts"
                </h2>
                <table class="w-full text-sm">
                    <tbody>
                        {keybindings
                            .into_iter()
                            .map(|(key, desc)| {
                                view! {
                                    <tr class="border-b border-border/30 \
                                               last:border-b-0">
                                        <td class="py-2 pr-4 text-text-secondary \
                                                   font-mono whitespace-nowrap">
                                            {key}
                                        </td>
                                        <td class="py-2 text-text-primary">
                                            {desc}
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </Modal>
    }
}
