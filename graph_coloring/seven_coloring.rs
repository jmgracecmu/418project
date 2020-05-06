pub mod seven_coloring{
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    pub enum Color{
        Red,
        Blue,
        Green,
        Yellow,
        Black,
        White,
        Pink,

    }
    impl Color{
        pub fn num_colors() -> usize{
            7
        }

        pub fn vector_of_colors() -> Vec<Color>{
            vec![Color::Red, Color::Blue, Color::Green, Color::Yellow, Color::Black, Color::White,
            Color::Pink]
        }
    }
}
