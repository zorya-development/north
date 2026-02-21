# Commit Message Template
# Use this format for commits that should appear in release notes

# Format:
# <type>: <short summary>
#
# User-facing: <what users will notice>
#
# Technical details:
# - Implementation detail 1
# - Implementation detail 2

# Example:
# feat: add inline task creation
#
# User-facing: Create tasks directly in the list without opening a modal.
# After creating the first task, the input chains below for rapid entry.
#
# Technical details:
# - Remove TaskCreateModal container and store
# - Add CreateTop mode to TraversableTaskList
# - Add TtlHandle for imperative start_create_top()
# - Fix blur handling with mode snapshot

# Types:
#   feat:     New user-facing feature
#   fix:      Bug fix users would notice
#   perf:     Performance improvement users would notice
#   refactor: Internal refactor (auto-hidden in release notes)
#   docs:     Documentation only
#   test:     Test only
#   chore:    Build/tooling changes

# If breaking change, add:
# BREAKING CHANGE: <description>
