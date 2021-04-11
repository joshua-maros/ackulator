make unit_class called Length
make base_unit called Meter, Meters {
    class: Length,
    symbol: "m",
    metric,
}
make derived_unit called Foot, Feet {
    symbol: "ft",
    value: 0.3048 * Meters,
}

make label called Pi, \pi for 3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679

make entity_class called Round
make entity_class called Circle

for any Entity 
where 
    Entity isa Circle
    R is Entity.Radius
conclude
    Entity is Round
    R isa Length
    Entity.Diameter isa Length

make law called AreaOfACircle
for any Circle
where
    // Commas optional, newlines serve the same purpose.
    Circle isa Circle
    r is Circle.Radius
    A is Circle.Area
conclude
    A = \pi * r ^ 2

// If you're allergic to multi-line statements...
make law called AreaOfACircle for any Circle where Circle isa Circle, r is Circle.Radius, A is Circle.Area conclude A = \pi * r ^ 2

make value called MyPizza {
    Circle,
    Radius: 0.1 * Meters,
}

check MyPizza isa Circle, MyPizza is Round

find MyPizza.Area;
// Or alternatively...
find MyPizza.Area using AreaOfACircle;
// Or alternatively...
find MyPizza.Area using AreaOfACircle where Circle is MyPizza;

check MyPizza.Area = \pi * 0.01 * Meters^2

make unit_class called Mass
make base_unit called Gram, Grams {
    class: Mass,
    symbol: "g",
    metric,
}
check 1 * Kilogram = 1000 * Grams

make unit_class called Time
make base_unit called Second, Seconds {
    class: Time,
    symbol: "s",
    partial_metric,
}
make derived_unit called Minute, Minutes {
    symbol: "m",
    value: 60 * Seconds,
}
make derived_unit called Hour, Hours {
    symbol: "h",
    value: 60 * Minutes,
}
make derived_unit called Day, Days {
    symbol: "d",
    value: 24 * Hours,
}
make derived_unit called Year, Years {
    symbol: "y",
    value: 365.24219 * Days,
}

make label called Velocity for Length / Time
make label called Acceleration for Velocity / Time
make label called Force for Acceleration / Mass

make derived_unit called Newton, Newtons {
    symbol: "N",
    value: 1 * Meters / Second ^ 2 / Kilogram,
    metric,
}

make label called Impulse for Force * Time
check Impulse = Velocity / Mass
