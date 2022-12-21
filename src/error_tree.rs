use itertools::Itertools;

#[derive(Debug)]
pub enum ErrorTree<L, E> {
    Leaf(E),
    Edge(L, Box<ErrorTree<L, E>>),
    Vec(Vec<ErrorTree<L, E>>),
}

pub trait ErrorLeaf {}

impl<L, E> From<Vec<E>> for ErrorTree<L, E>
where
    E: Into<ErrorTree<L, E>>,
{
    fn from(errors: Vec<E>) -> Self {
        Self::Vec(errors.into_iter().map(|e| e.into()).collect_vec())
    }
}

impl<L, E> From<E> for ErrorTree<L, E>
where
    E: ErrorLeaf,
{
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

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[derive(Debug)]
    struct Error(String);

    impl ErrorLeaf for Error {}

    fn faulty(error: &str) -> Result<(), Error> {
        Err(Error(error.into()))
    }

    #[test]
    fn can_build_tree_from_vec_of_errors() -> Result<(), Error> {
        let error1 = faulty("error1");
        let error2 = faulty("error2");

        let (_, errors): (Vec<_>, Vec<_>) = vec![error1, error2].into_iter().partition_result();

        let tree: ErrorTree<&'static str, _> = errors.into();

        let _flat_errors = tree.flatten_tree();

        Ok(())
    }
}
