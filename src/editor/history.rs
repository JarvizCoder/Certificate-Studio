use crate::model::project::CertificateProject;

/// Manages undo and redo state snapshots for the certificate project.
#[derive(Debug, Clone)]
pub struct ProjectHistory {
    past: Vec<CertificateProject>,
    future: Vec<CertificateProject>,
    max_history: usize,
}

impl Default for ProjectHistory {
    fn default() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            max_history: 50,
        }
    }
}

impl ProjectHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Save current state before a mutation.
    pub fn push_snapshot(&mut self, current: &CertificateProject) {
        if let Some(last) = self.past.last() {
            if last == current {
                return;
            }
        }
        self.past.push(current.clone());
        if self.past.len() > self.max_history {
            self.past.remove(0);
        }
        self.future.clear();
    }

    /// Undo to previous project state.
    pub fn undo(&mut self, current: &mut CertificateProject) -> bool {
        if let Some(prev) = self.past.pop() {
            self.future.push(current.clone());
            *current = prev;
            true
        } else {
            false
        }
    }

    /// Redo to next project state.
    pub fn redo(&mut self, current: &mut CertificateProject) -> bool {
        if let Some(next) = self.future.pop() {
            self.past.push(current.clone());
            *current = next;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::element::TextElement;

    #[test]
    fn test_undo_redo_lifecycle() {
        let mut history = ProjectHistory::new();
        let mut project = CertificateProject::default();
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        // Snapshot initial state
        history.push_snapshot(&project);

        // Mutate project: add element
        let elem = TextElement::new_default("Title", "Certificate", [10.0, 10.0]);
        project.add_element(elem);
        assert_eq!(project.elements.len(), 1);

        // Snapshot state with 1 element
        history.push_snapshot(&project);

        // Mutate again: add second element
        let elem2 = TextElement::new_default("Name", "John Doe", [10.0, 30.0]);
        project.add_element(elem2);
        assert_eq!(project.elements.len(), 2);

        // Undo back to 1 element
        assert!(history.can_undo());
        assert!(history.undo(&mut project));
        assert_eq!(project.elements.len(), 1);
        assert_eq!(project.elements[0].name, "Title");
        assert!(history.can_redo());

        // Undo back to 0 elements
        assert!(history.undo(&mut project));
        assert_eq!(project.elements.len(), 0);

        // Redo back to 1 element
        assert!(history.redo(&mut project));
        assert_eq!(project.elements.len(), 1);
        assert_eq!(project.elements[0].name, "Title");

        // Redo back to 2 elements
        assert!(history.redo(&mut project));
        assert_eq!(project.elements.len(), 2);
    }
}

