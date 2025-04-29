use super::Action;

pub fn transform_actions(actions: &mut [Action], history: &[Action]) {
    for action in actions {
        match action {
            Action::Insertion { from, owner, .. } => *from = transform_index(*from, owner, history),
            Action::Deletion { from, to, owner } => {
                let len = *to - *from;
                *from = transform_index(*from, owner, history);
                *to = *from + len;
            }
        }
    }
}

pub fn transform_index(idx: usize, idx_owner: &String, history: &[Action]) -> usize {
    let mut transformed = idx;

    for action in history {
        match action {
            // Ignore self actions
            Action::Deletion { owner, .. } | Action::Insertion { owner, .. }
                if owner == idx_owner =>
            {
                continue
            }

            // All deletion is left to idx, remove range
            Action::Deletion { from, to, .. } if *to < transformed => transformed -= to - from,
            // All deletion is right to idx, ignore it
            Action::Deletion { .. } => continue,

            // Insertion is left to idx, add offset
            Action::Insertion { from, text, .. } if *from <= transformed => {
                transformed += text.len()
            }
            // Insertion is right to idx, ignore it
            Action::Insertion { .. } => continue,
        }
    }

    transformed
}

pub fn apply_actions(text: impl Into<String>, actions: &[Action]) -> String {
    let mut output: String = text.into();

    for action in actions {
        match action {
            Action::Insertion { from, text, .. } if *from < output.len() => {
                output.insert_str(*from, text)
            }
            Action::Deletion { from, to, .. } if *to < output.len() => {
                output.drain(from..to);
            }
            // Handle out of bounds action
            Action::Insertion { text, .. } => output.push_str(text),
            Action::Deletion { from, to, .. } => {
                output.drain((output.len().saturating_sub(to - from))..);
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_transformations() {
        let owner = "owner_a".to_owned();

        // "ac" -> "abc"
        let history = &[Action::Insertion {
            from: 1,
            text: "b".to_owned(),
            owner: owner.clone(),
        }];

        let owner = "owner_b".to_owned();
        let text = "abc";

        // "ac" -> "acde"
        let actions = &mut [Action::Insertion {
            from: 2,
            text: "de".to_owned(),
            owner: owner.clone(),
        }];

        transform_actions(actions, history);

        let output = apply_actions(text, actions);
        let expected = "abcde";

        dbg!(&history);
        dbg!(&actions);
        assert_eq!(output, expected);
    }

    #[test]
    fn complex_transformations() {
        let owner = "owner_a".to_owned();

        // "cd" -> "ad"
        // Inserts "a" at 0
        // Deletes from 1 to 2
        // "cd" -> "bcde" -> "abce"
        // Inserts "b" at 0 -> at 1
        // Inserts "e" at 3 -> at 3

        // "cd" -> "ac"
        let history = &[
            Action::Insertion {
                from: 0,
                text: "a".to_owned(),
                owner: owner.clone(),
            },
            Action::Deletion {
                from: 1,
                to: 2,
                owner: owner.clone(),
            },
        ];

        let owner = "owner_b".to_owned();
        let text = "ad";

        // "cd" -> "bcde"
        let actions = &mut [
            Action::Insertion {
                from: 0,
                text: "b".to_owned(),
                owner: owner.clone(),
            },
            Action::Insertion {
                from: 3,
                text: "e".to_owned(),
                owner: owner.clone(),
            },
        ];

        transform_actions(actions, history);

        let output = apply_actions(text, actions);
        let expected = "abde";

        dbg!(&history);
        dbg!(&actions);
        assert_eq!(output, expected);
    }

    #[test]
    fn simple_apply_actions() {
        let owner = "owner".to_owned();
        let text = "abd";
        let actions = &[
            Action::Deletion {
                from: 2,
                to: 3,
                owner: owner.clone(),
            },
            Action::Insertion {
                from: 2,
                text: "c".to_owned(),
                owner: owner.clone(),
            },
        ];

        let output = apply_actions(text, actions);
        let expected = "abc";

        assert_eq!(output, expected);
    }

    #[test]
    fn complex_apply_actions() {
        let owner = "owner".to_owned();
        let text = "abc";
        let actions = &[
            Action::Insertion {
                from: 3,
                text: "D".to_owned(),
                owner: owner.clone(),
            },
            Action::Deletion {
                from: 1,
                to: 4,
                owner: owner.clone(),
            },
            Action::Insertion {
                from: 1,
                text: "BC".to_owned(),
                owner: owner.clone(),
            },
            Action::Deletion {
                from: 0,
                to: 1,
                owner: owner.clone(),
            },
            Action::Insertion {
                from: 0,
                text: "A".to_owned(),
                owner: owner.clone(),
            },
        ];

        let output = apply_actions(text, actions);
        let expected = "ABC";

        assert_eq!(output, expected);
    }
}
