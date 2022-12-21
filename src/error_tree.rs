use itertools::Itertools;

#[derive(Debug)]
pub enum ErrorTree<L, E> {
    Leaf(E),
    Edge(L, Box<ErrorTree<L, E>>),
    Vec(Vec<ErrorTree<L, E>>),
}

impl<L, E> From<Vec<E>> for ErrorTree<L, E>
where
    E: Into<ErrorTree<L, E>>,
{
    fn from(errors: Vec<E>) -> Self {
        Self::Vec(errors.into_iter().map(|e| e.into()).collect_vec())
    }
}

impl<L, E> From<Vec<ErrorTree<L, E>>> for ErrorTree<L, E>
where
    E: Into<ErrorTree<L, E>>,
{
    fn from(errors: Vec<ErrorTree<L, E>>) -> Self {
        Self::Vec(errors)
    }
}

impl<L, E> From<E> for ErrorTree<L, E> {
    fn from(error: E) -> Self {
        Self::Leaf(error)
    }
}

#[derive(Debug)]
pub struct FlatError<L, E> {
    pub path: Vec<L>,
    pub error: E,
}

impl<L, E> ErrorTree<L, E> {
    pub fn flatten_tree(&self) -> Vec<FlatError<&L, &E>> {
        match self {
            ErrorTree::Leaf(error) => vec![FlatError {
                path: Vec::new(),
                error,
            }],
            ErrorTree::Edge(label, tree) => {
                let mut flat_errors = tree.flatten_tree();
                for flat in &mut flat_errors {
                    flat.path.push(&label);
                }
                flat_errors
            }
            ErrorTree::Vec(errors) => errors
                .into_iter()
                .flat_map(|tree| tree.flatten_tree())
                .collect_vec(),
        }
    }
}

trait LabelError<L, E>
where
    E: Into<ErrorTree<L, E>>,
{
    fn label_error(self, label: L) -> ErrorTree<L, E>;
}

trait LabelResult<T, L, E>
where
    E: Into<ErrorTree<L, E>>,
{
    fn label_result(self, label: L) -> Result<T, ErrorTree<L, E>>;
}

impl<L, E> LabelError<L, E> for E {
    fn label_error(self, label: L) -> ErrorTree<L, E> {
        ErrorTree::Edge(label, Box::new(self.into()))
    }
}

impl<L, E, T> LabelResult<T, L, E> for Result<T, E> {
    fn label_result(self, label: L) -> Result<T, ErrorTree<L, E>> {
        self.map_err(|e| e.label_error(label))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[derive(Debug)]
    struct Error(String);

    fn faulty(error: &str) -> Result<(), Error> {
        Err(Error(error.into()))
    }

    #[test]
    fn can_build_tree_from_vec_of_errors() -> Result<(), Error> {
        let error1 = faulty("error1").label_result("label1");
        let error2 = faulty("error2").label_result("label2");

        let (_, errors): (Vec<_>, Vec<_>) = vec![error1, error2].into_iter().partition_result();

        let tree: ErrorTree<&'static str, _> = errors.label_error("parent_label");

        let flat_errors = tree.flatten_tree();

        assert!(false, "{:#?}", flat_errors);

        Ok(())
    }
}
