namespace Kata {
    function WinCondition (x : Bool, y : Bool, a : Bool, b : Bool) : Bool {
        return (x and y) == (a != b);
    }
}
