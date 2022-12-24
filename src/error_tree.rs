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

pub trait LabelResult<T, L, E> {
    fn label_error(self, label: L) -> Result<T, ErrorTree<L, E>>;
}

impl<T, L, E> LabelResult<T, L, E> for Result<T, E>
where
    ErrorTree<L, E>: From<E>,
{
    fn label_error(self, label: L) -> Result<T, ErrorTree<L, E>> {
        self.map_err(|e| {
            let tree: ErrorTree<L, E> = e.into();
            tree.into_tree_with_label(label)
        })
    }
}

impl<T, L, E> LabelResult<T, L, E> for Result<T, ErrorTree<L, E>> {
    fn label_error(self, label: L) -> Result<T, ErrorTree<L, E>> {
        self.map_err(|tree| tree.into_tree_with_label(label))
    }
}

pub trait FlattenResultErrors<T, L, E> {
    fn flatten_results(self) -> Result<T, Vec<FlatError<L, E>>>;
}

impl<T, L, E> FlattenResultErrors<T, L, E> for Result<T, ErrorTree<L, E>>
where
    L: Clone,
{
    fn flatten_results(self) -> Result<T, Vec<FlatError<L, E>>> {
        self.map_err(|tree| tree.flatten_tree())
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
    fn can_build_tree_from_vec_of_results() {
        let result_1 = faulty("error1").map_err(|e| e.into_tree_with_label("label1"));
        let result_2 = faulty("error2").map_err(|e| e.into_tree_with_label("label2"));

        let (_, errors): (Vec<_>, Vec<_>) = vec![result_1, result_2].into_iter().partition_result();

        let tree: ErrorTree<&'static str, Error> = errors.into();
        let tree = tree.into_tree_with_label("parent_label");

        let flat_errors = tree.flatten_tree();

        assert!(
            matches!(
                &flat_errors[..],
                [
                    FlatError {
                        path: path1,
                        error: Error(error1),
                    },
                    FlatError {
                        path: path2,
                        error: Error(error2),
                    },
                ]
                if path1 == &vec!["label1", "parent_label"]
                && path2 == &vec!["label2", "parent_label"]
                && error1 == "error1"
                && error2 == "error2"
            ),
            "unexpected: {:#?}",
            flat_errors
        );
    }

    #[test]
    fn can_call_into_result_from_vec_of_results() {
        let result_1 = faulty("error1").map_err(|e| e.into_tree_with_label("label1"));
        let result_2 = faulty("error2").map_err(|e| e.into_tree_with_label("label2"));

        let result: Result<(), _> = vec![result_1, result_2]
            .into_iter()
            .partition_result()
            .into_result();

        let flat_result = result.map_err(|e| e.flatten_tree());

        let flat_errors = flat_result.unwrap_err();

        assert!(
            matches!(
                &flat_errors[..],
                [
                    FlatError {
                        path: path1,
                        error: Error(error1),
                    },
                    FlatError {
                        path: path2,
                        error: Error(error2),
                    },
                ]
                if path1 == &vec!["label1"]
                && path2 == &vec!["label2"]
                && error1 == "error1"
                && error2 == "error2"
            ),
            "unexpected: {:#?}",
            flat_errors
        );
    }

    #[test]
    fn can_call_into_result_from_vec_of_errors() {
        let error1 = Error("error1".into()).into_tree_with_label("label1");
        let error2 = Error("error2".into()).into_tree_with_label("label2");

        let result = vec![error1, error2].into_result();

        let flat_result = result.map_err(|e| e.flatten_tree());

        let flat_errors = flat_result.unwrap_err();

        assert!(
            matches!(
                &flat_errors[..],
                [
                    FlatError {
                        path: path1,
                        error: Error(error1),
                    },
                    FlatError {
                        path: path2,
                        error: Error(error2),
                    },
                ]
                if path1 == &vec!["label1"]
                && path2 == &vec!["label2"]
                && error1 == "error1"
                && error2 == "error2"
            ),
            "unexpected: {:#?}",
            flat_errors
        );
    }

    // For the README
    // A function that returns an error
    fn faulty_function() -> Result<(), Error> {
        Err(Error("error".into()))
    }

    // A function that returns more than one error
    fn parent_function() -> Result<Vec<()>, ErrorTree<&'static str, Error>> {
        let result1 = faulty_function().label_error("first faulty");
        let result2 = faulty_function().label_error("second faulty");

        vec![result1, result2]
            .into_iter()
            .partition_result()
            .into_result()
            .label_error("parent function")
    }

    // your main function
    #[test]
    fn main_function() {
        let result = parent_function();

        let flat_results = result.flatten_results();
        let flat_errors: Vec<FlatError<&str, Error>> = flat_results.unwrap_err();

        assert!(
            matches!(
                &flat_errors[..],
                [
                    FlatError {
                        path: path1,
                        error: Error(_),
                    },
                    FlatError {
                        path: path2,
                        error: Error(_),
                    },
                ]
                if path1 == &vec!["first faulty", "parent function"]
                && path2 == &vec!["second faulty", "parent function"]
            ),
            "unexpected: {:#?}",
            flat_errors
        );
    }
}
