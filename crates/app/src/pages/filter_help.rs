use leptos::prelude::*;

#[component]
pub fn FilterHelpPage() -> impl IntoView {
    view! {
        <div class="max-w-2xl space-y-6">
            <h1 class="text-2xl font-semibold tracking-tight text-text-primary">
                "Filter Query Syntax"
            </h1>

            <Section title="Fields">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="text-left text-text-secondary border-b border-border">
                            <th class="py-1.5 pr-4">"Field"</th>
                            <th class="py-1.5">"Description"</th>
                        </tr>
                    </thead>
                    <tbody class="text-text-primary">
                        <FieldRow field="title" desc="Task title"/>
                        <FieldRow field="body" desc="Task body/description"/>
                        <FieldRow
                            field="project"
                            desc="Project name (resolves by title)"
                        />
                        <FieldRow
                            field="tags (or tag)"
                            desc="Tag name"
                        />
                        <FieldRow
                            field="status"
                            desc="COMPLETED/DONE or ACTIVE/OPEN"
                        />
                        <FieldRow
                            field="due_date (or due)"
                            desc="Due date (YYYY-MM-DD)"
                        />
                        <FieldRow
                            field="start_at (or start)"
                            desc="Start date/time"
                        />
                        <FieldRow
                            field="column (or col)"
                            desc="Project status/column name"
                        />
                        <FieldRow
                            field="created (or created_at)"
                            desc="Creation timestamp"
                        />
                        <FieldRow
                            field="updated (or updated_at)"
                            desc="Last update timestamp"
                        />
                    </tbody>
                </table>
            </Section>

            <Section title="Operators">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="text-left text-text-secondary border-b border-border">
                            <th class="py-1.5 pr-4">"Operator"</th>
                            <th class="py-1.5">"Example"</th>
                        </tr>
                    </thead>
                    <tbody class="text-text-primary font-mono">
                        <OpRow op="=" example="status = 'ACTIVE'"/>
                        <OpRow op="!=" example="status != 'COMPLETED'"/>
                        <OpRow op="=~" example="tags =~ 'work:*'"/>
                        <OpRow op="!~" example="title !~ '*draft*'"/>
                        <OpRow op=">" example="due_date > '2024-01-01'"/>
                        <OpRow op="<" example="created < '2024-06-01'"/>
                        <OpRow op=">=" example="due_date >= '2024-01-01'"/>
                        <OpRow op="<=" example="due_date <= '2024-12-31'"/>
                        <OpRow op="is null" example="due_date is null"/>
                        <OpRow op="is not null" example="project is not null"/>
                        <OpRow op="in [...]" example="tags in ['work', 'urgent']"/>
                        <OpRow
                            op="not in [...]"
                            example="tags not in ['archive']"
                        />
                    </tbody>
                </table>
            </Section>

            <Section title="Logical Operators">
                <ul class="list-disc list-inside text-sm text-text-primary space-y-1">
                    <li>
                        <code class="text-accent">"AND"</code>
                        " \u{2014} both conditions must match"
                    </li>
                    <li>
                        <code class="text-accent">"OR"</code>
                        " \u{2014} either condition matches"
                    </li>
                    <li>
                        <code class="text-accent">"NOT"</code>
                        " \u{2014} negates a condition"
                    </li>
                    <li>
                        "Parentheses "
                        <code class="text-accent">"( )"</code>
                        " for grouping"
                    </li>
                </ul>
            </Section>

            <Section title="Glob Patterns">
                <ul class="list-disc list-inside text-sm text-text-primary space-y-1">
                    <li>
                        <code class="text-accent">"*"</code>
                        " matches any number of characters"
                    </li>
                    <li>
                        <code class="text-accent">"?"</code>
                        " matches a single character"
                    </li>
                </ul>
            </Section>

            <Section title="ORDER BY">
                <p class="text-sm text-text-primary">
                    "Append "
                    <code class="text-accent">"ORDER BY field ASC|DESC"</code>
                    " to sort results."
                </p>
                <pre class="mt-1 text-sm font-mono text-text-secondary \
                            bg-bg-tertiary rounded px-3 py-2">
                    "status = 'ACTIVE' ORDER BY due_date ASC"
                </pre>
            </Section>

            <Section title="Examples">
                <div class="space-y-2">
                    <Example
                        label="Active tasks with work tags"
                        query="status = 'ACTIVE' AND tags =~ 'work:*'"
                    />
                    <Example
                        label="Overdue tasks"
                        query="due_date < '2024-01-01' AND status != 'COMPLETED'"
                    />
                    <Example
                        label="Inbox tasks (no project)"
                        query="project is null AND status = 'ACTIVE'"
                    />
                    <Example
                        label="Tasks in specific projects"
                        query="project in ['Alpha', 'Beta'] ORDER BY created DESC"
                    />
                    <Example
                        label="Untagged tasks"
                        query="tags is null AND status = 'ACTIVE'"
                    />
                </div>
            </Section>
        </div>
    }
}

#[component]
fn Section(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div>
            <h2 class="text-sm font-medium text-text-secondary \
                        uppercase tracking-wide mb-2">
                {title}
            </h2>
            {children()}
        </div>
    }
}

#[component]
fn FieldRow(field: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <tr class="border-b border-border/50">
            <td class="py-1.5 pr-4 font-mono text-accent">{field}</td>
            <td class="py-1.5 text-text-secondary">{desc}</td>
        </tr>
    }
}

#[component]
fn OpRow(op: &'static str, example: &'static str) -> impl IntoView {
    view! {
        <tr class="border-b border-border/50">
            <td class="py-1.5 pr-4 text-accent">{op}</td>
            <td class="py-1.5 text-text-secondary">{example}</td>
        </tr>
    }
}

#[component]
fn Example(label: &'static str, query: &'static str) -> impl IntoView {
    view! {
        <div>
            <p class="text-xs text-text-secondary">{label}</p>
            <pre class="text-sm font-mono text-text-primary \
                        bg-bg-tertiary rounded px-3 py-1.5 mt-0.5">
                {query}
            </pre>
        </div>
    }
}
