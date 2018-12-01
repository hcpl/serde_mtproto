pub(crate) trait IteratorResultExt<T, E>: Iterator<Item = Result<T, E>> {
    fn collect_results(self) -> Result<Vec<T>, Vec<E>>;
}

impl<I, T, E> IteratorResultExt<T, E> for I
where
    I: Iterator<Item = Result<T, E>>,
{
    fn collect_results(self) -> Result<Vec<T>, Vec<E>> {
        let mut items = Vec::with_capacity(self.size_hint().0);
        let mut errors = Vec::new();

        for res in self {
            match res {
                Ok(item) => items.push(item),
                Err(error) => errors.push(error),
            }
        }

        match errors.len() {
            0 => Ok(items),
            _ => Err(errors),
        }
    }
}
