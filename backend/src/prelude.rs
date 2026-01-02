pub trait IteratorExtension<R, E>: Iterator<Item = Result<R, E>> {
    fn try_collect_vec(self) -> Result<Vec<R>, E>
    where
        Self: Sized,
    {
        let mut vec = Vec::new();

        let (_, hint_max) = self.size_hint();
        if let Some(hint_max) = hint_max {
            vec.reserve_exact(hint_max);
        }

        for item in self {
            vec.push(item?);
        }

        Ok(vec)
    }
}

impl<R, E, T: Iterator<Item = Result<R, E>>> IteratorExtension<R, E> for T {}
