pub mod three_coloring{
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    pub enum Color{
        Red,
        Blue,
        Green,
    }
    impl Color{
        pub fn num_colors() -> usize{
            3
        }

        pub fn vector_of_colors() -> Vec<Color>{
            vec![Color::Red, Color::Blue, Color::Green]
        }
    }
}
