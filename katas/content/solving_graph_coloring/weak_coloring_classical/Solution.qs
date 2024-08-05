namespace Kata {
    function IsWeakColoringValid(V : Int, edges: (Int, Int)[], colors: Int[]) : Bool {
        for vertex in 0 .. V - 1 {
            mutable neighborCount = 0;
            mutable hasDifferentNeighbor = false;

            for (start, end) in edges {
                if start == vertex or end == vertex {
                    set neighborCount += 1;
                    if colors[start] != colors[end] {
                        set hasDifferentNeighbor = true;
                    }
                }
            }

            if neighborCount > 0 and not hasDifferentNeighbor {
                return false;
            }
        }
        return true;
    }
}
