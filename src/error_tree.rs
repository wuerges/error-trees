use itertools::Itertools;

#[derive(Debug)]
pub enum ErrorTree<L, E> {
    Leaf(E),
    Edge(L, Box<ErrorTree<L, E>>),
    Vec(Vec<ErrorTree<L, E>>),
}

impl<L, E> ErrorTree<L, E> {
    pub fn leaf(e: E) -> Self {
        Self::Leaf(e)
    }
}

#[derive(Debug)]
pub struct FlatError<L, E> {
    pub path: Vec<L>,
    pub error: E,
}

impl<L, E> ErrorTree<L, E>
where
    L: Clone,
{
    pub fn flatten_tree(self) -> Vec<FlatError<L, E>> {
        match self {
            ErrorTree::Leaf(error) => vec![FlatError {
                path: Vec::new(),
                error,
            }],
            ErrorTree::Edge(label, tree) => {
                let mut flat_errors = tree.flatten_tree();
                for flat in &mut flat_errors {
                    flat.path.push(label.clone());
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

trait IntoErrorTree<L, E> {
    fn into_tree_with_label(self, label: L) -> ErrorTree<L, E>;
}

impl<L, E> IntoErrorTree<L, E> for E
where
    E: Into<ErrorTree<L, E>>,
{
    fn into_tree_with_label(self, label: L) -> ErrorTree<L, E> {
        ErrorTree::Edge(label, Box::new(self.into()))
    }
}

impl<L, E> IntoErrorTree<L, E> for ErrorTree<L, E> {
    fn into_tree_with_label(self, label: L) -> ErrorTree<L, E> {
        ErrorTree::Edge(label, Box::new(self))
    }
}

impl<L, E> From<Vec<ErrorTree<L, E>>> for ErrorTree<L, E> {
    fn from(subtrees: Vec<ErrorTree<L, E>>) -> Self {
        ErrorTree::Vec(subtrees)
    }
}

impl<L, E> From<Vec<E>> for ErrorTree<L, E>
where
    E: IntoErrorTree<L, E>,
    ErrorTree<L, E>: From<E>,
{
    fn from(errors: Vec<E>) -> Self {
        ErrorTree::Vec(errors.into_iter().map(|x| x.into()).collect_vec())
    }
}

pub trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl<T, E> IntoResult<T, E> for (T, Vec<E>)
where
    Vec<E>: Into<E>,
{
    fn into_result(self) -> Result<T, E> {
        let (oks, errs) = self;

        if errs.is_empty() {
            Ok(oks)
        } else {
            Err(errs.into())
        }
    }
}

impl<E> IntoResult<(), E> for Vec<E>
where
    Vec<E>: Into<E>,
{
    fn into_result(self) -> Result<(), E> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[derive(Debug)]
    struct Error(String);

    impl<L> From<Error> for ErrorTree<L, Error> {
        fn from(e: Error) -> Self {
            Self::leaf(e)
        }
    }

    fn faulty(error: &str) -> Result<(), Error> {
        Err(Error(error.into()))
    }

    #[test]
    fn can_build_tree_from_vec_of_results() -> Result<(), Error> {
        let result_1 = faulty("error1").map_err(|e| e.into_tree_with_label("label1"));
        let result_2 = faulty("error2").map_err(|e| e.into_tree_with_label("label2"));

        let (_, errors): (Vec<_>, Vec<_>) = vec![result_1, result_2].into_iter().partition_result();

        let tree: ErrorTree<&'static str, Error> = errors.into();
        let tree = tree.into_tree_with_label("parent_label");

        let flat_errors = tree.flatten_tree();

        assert!(false, "{:#?}", flat_errors);

        Ok(())
    }

    #[test]
    fn can_call_into_result_from_vec_of_results() -> Result<(), Error> {
        let result_1 = faulty("error1").map_err(|e| e.into_tree_with_label("label1"));
        let result_2 = faulty("error2").map_err(|e| e.into_tree_with_label("label2"));

        let result: Result<(), _> = vec![result_1, result_2]
            .into_iter()
            .partition_result()
            .into_result();

        let flat_result = result.map_err(|e| e.flatten_tree());

        assert!(false, "{:#?}", flat_result);

        Ok(())
    }

    #[test]
    fn can_call_into_result_from_vec_of_errors() -> Result<(), Error> {
        let error1 = Error("error1".into()).into_tree_with_label("label1");
        let error2 = Error("error2".into()).into_tree_with_label("label2");

        let result = vec![error1, error2].into_result();

        let flat_result = result.map_err(|e| e.flatten_tree());

        assert!(false, "{:#?}", flat_result);

        Ok(())
    }
}
