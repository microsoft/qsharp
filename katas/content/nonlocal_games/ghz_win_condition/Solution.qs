namespace Kata {
    function WinCondition (rst : Bool[], abc : Bool[]) : Bool {
        return (rst[0] or rst[1] or rst[2]) == (abc[0] != abc[1] != abc[2]);
    }
}
