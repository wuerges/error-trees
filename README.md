# Error Trees

Fail in a spectacular manner with multiple errors, instead only a single one!

```rust
// The error type
#[derive(Debug)]
struct Error(String);

impl<L> From<Error> for ErrorTree<L, Error> {
    fn from(e: Error) -> Self {
        Self::leaf(e)
    }
}

// A function that returns an error
fn faulty_function() -> Result<(), Error> {
    Err(Error("error".into()))
}

// A function that returns more than one error
fn parent_function() -> Result<Vec<()>, ErrorTree<&'static str, Error>> {
    let result1 = faulty_function().label_error("first faulty");
    let result2 = faulty_function().label_error("second faulty");

    let result: Result<_, ErrorTree<_, _>> = vec![result1, result2]
        .into_iter()
        .partition_result::<Vec<_>, Vec<_>, _, _>()
        .into_result();
    result.label_error("parent function")
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
```
