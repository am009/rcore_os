
struct Person<'a> {
    name: &'a str,
    age: i32,
    height: i32,
    weight: i32
}

impl<'a> Person<'a> {
    fn new(name: &'a str, age: i32, height: i32, weight: i32) -> Self {
        Person{ name: name, age: age, height: height, weight: weight }
    }

    fn print(self: &Self) {
        println!("Name: {}", self.name);
        println!("\tAge: {}", self.age);
        println!("\tHeight: {}", self.height);
        println!("\tWeight: {}", self.weight);
    }
}

fn main() {
    let mut joe = Person::new("Joe Alex", 32, 64, 140);
    let mut frank = Person::new("Frank Blank", 20, 72, 180);

    println!("Joe is at memory location: {:p}", (&joe as *const Person));
    joe.print();

    println!("Frank is at memory location: {:p}", (&frank as *const Person));
    frank.print();

    joe.age = joe.age + 20;
    joe.height = joe.height - 2;
    joe.weight = joe.weight + 40;
    joe.print();

    frank.age = frank.age + 20;
    frank.weight = frank.weight + 20;
    frank.print();
}