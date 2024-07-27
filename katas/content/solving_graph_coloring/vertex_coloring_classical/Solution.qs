namespace Kata {
    function IsVertexColoringValid(V : Int, edges: (Int, Int)[], colors: Int[]) : Bool {
        for (v0, v1) in edges {
            if colors[v0] == colors[v1] {
                return false;
            }
        }
        return true;
    }
}
