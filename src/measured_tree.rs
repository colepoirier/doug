/// HackerFoo
///  —
/// https://discord.com/channels/691052431525675048/905143227613589505/905265575536758804
/// Today at 6:22 PM PDT
/// My implementation is a binary tree with a bounding volume in each node, and on the leaves. I made up the name, so you probably won't find anything on it, but I wouldn't be surprised if it's similar or identical to something published.
/// I call it a measured tree because I have a trait called "Measured" with a method fn measure(&self) -> V where V is a monoid.
/// So it's this:

/// pub enum MeasuredTree<V, A> {
///     Node { measure: V,
///            left: Box<Self>,
///            right: Box<Self> },
///     Leaf { measure: V,
///            val: A },
///     Empty
/// }

/// The values A in each Leaf implement Measure, and it's trivial to implement Measure for the MeasuredTree as well.

pub trait Measured {
    fn measure(&self) -> V // where V is a monoid
    {
    }
}

pub enum MeasuredTree<V, A> {
    Node {
        measure: V,
        left: Box<Self>,
        right: Box<Self>,
    },
    Leaf {
        measure: V,
        val: A,
    },
    Empty,
}
