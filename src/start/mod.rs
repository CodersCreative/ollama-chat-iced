#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Section<'a>{
    pub title: &'a str,
    pub prompts : [&'a str; 4]
}

pub const SECTIONS : [Section; 5] = [
    Section{
        title : "General",
        prompts : [
            "How does AI work",
            "Are white holes real",
            "How many Ts in tomato",
            "List of questions to ask an AI"
        ]
    },
    Section{
        title : "Create",
        prompts : [
            "Write a short story about a robot",
            "Outline a sci-fi novel about robots",
            "Create a character profile for a robot",
            "Give me 5 tips to create a story"
        ]
    },
    Section{
        title : "Explore",
        prompts : [
            "Good books by African authors",
            "Countries ranked by crime rate",
            "Richest people in the world",
            "Advantages of Linux over Windows"
        ]
    },
    Section{
        title : "Code",
        prompts : [
            "Write code to invert a binary search tree in golang",
            "Write html for a privacy policy",
            "Write a recursive factorial function in python",
            "Write code to parse a struct to json in Rust"
        ]
    },
    Section{
        title : "Learn",
        prompts : [
            "What is a garbage collector",
            "Best ways to learn to code",
            "Explain lifetimes in Rust",
            "Why is open-source better"
        ]
    },
];

