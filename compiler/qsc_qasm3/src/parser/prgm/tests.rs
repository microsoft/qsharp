// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::parse;
use crate::parser::tests::check;
use expect_test::expect;
const STDGATES_INC: &str = include_str!("stdgates.inc");

#[test]
fn stdgates_inc_can_be_parsed_correrctly() {
    check(
        parse,
        STDGATES_INC,
        &expect![[r#"
            Program [356-2413]:
                Stmt [356-393]
                    StmtKind: Gate [356-393]: Ident [361-362] "p"(Ident [363-365] "λ") Ident [367-368] "a"

                    Stmt [371-391]
                        StmtKind: GPhase [371-391]:
                        Expr [385-387]: Ident [385-387] "λ"
                        GateOperand IndexedIdent [389-390]: Ident [389-390] "a"[]
                Stmt [431-473]
                    StmtKind: Gate [431-473]: Ident [436-437] "x"(<no params>) Ident [438-439] "a"

                    Stmt [442-457]
                        StmtKind: GateCall [442-457]: Ident [442-443] "U"
                        Expr [444-446]: Ident [444-446] "π"
                        Expr [448-449]: Lit: Int(0)
                        Expr [451-453]: Ident [451-453] "π"
                        GateOperand IndexedIdent [455-456]: Ident [455-456] "a"[]
                    Stmt [458-472]
                        StmtKind: GPhase [458-472]:
                        Expr [465-470]: BinOp (Div):
                            Expr [465-468]: UnOp (Neg):
                                Expr [466-468]: Ident [466-468] "π"
                            Expr [469-470]: Lit: Int(2)
                Stmt [508-555]
                    StmtKind: Gate [508-555]: Ident [513-514] "y"(<no params>) Ident [515-516] "a"

                    Stmt [519-539]
                        StmtKind: GateCall [519-539]: Ident [519-520] "U"
                        Expr [521-523]: Ident [521-523] "π"
                        Expr [525-529]: BinOp (Div):
                            Expr [525-527]: Ident [525-527] "π"
                            Expr [528-529]: Lit: Int(2)
                        Expr [531-535]: BinOp (Div):
                            Expr [531-533]: Ident [531-533] "π"
                            Expr [534-535]: Lit: Int(2)
                        GateOperand IndexedIdent [537-538]: Ident [537-538] "a"[]
                    Stmt [540-554]
                        StmtKind: GPhase [540-554]:
                        Expr [547-552]: BinOp (Div):
                            Expr [547-550]: UnOp (Neg):
                                Expr [548-550]: Ident [548-550] "π"
                            Expr [551-552]: Lit: Int(2)
                Stmt [583-604]
                    StmtKind: Gate [583-604]: Ident [588-589] "z"(<no params>) Ident [590-591] "a"

                    Stmt [594-602]
                        StmtKind: GateCall [594-602]: Ident [594-595] "p"
                        Expr [596-598]: Ident [596-598] "π"
                        GateOperand IndexedIdent [600-601]: Ident [600-601] "a"[]
                Stmt [633-677]
                    StmtKind: Gate [633-677]: Ident [638-639] "h"(<no params>) Ident [640-641] "a"

                    Stmt [644-661]
                        StmtKind: GateCall [644-661]: Ident [644-645] "U"
                        Expr [646-650]: BinOp (Div):
                            Expr [646-648]: Ident [646-648] "π"
                            Expr [649-650]: Lit: Int(2)
                        Expr [652-653]: Lit: Int(0)
                        Expr [655-657]: Ident [655-657] "π"
                        GateOperand IndexedIdent [659-660]: Ident [659-660] "a"[]
                    Stmt [662-676]
                        StmtKind: GPhase [662-676]:
                        Expr [669-674]: BinOp (Div):
                            Expr [669-672]: UnOp (Neg):
                                Expr [670-672]: Ident [670-672] "π"
                            Expr [673-674]: Lit: Int(4)
                Stmt [714-742]
                    StmtKind: Gate [714-742]: Ident [719-720] "s"(<no params>) Ident [721-722] "a"

                    Stmt [725-740]
                        StmtKind: GateCall [725-740]: Ident [736-737] "z"
                        GateOperand IndexedIdent [738-739]: Ident [738-739] "a"[]
                Stmt [780-816]
                    StmtKind: Gate [780-816]: Ident [785-788] "sdg"(<no params>) Ident [789-790] "a"

                    Stmt [793-814]
                        StmtKind: GateCall [793-814]: Ident [810-811] "z"
                        GateOperand IndexedIdent [812-813]: Ident [812-813] "a"[]
                Stmt [839-867]
                    StmtKind: Gate [839-867]: Ident [844-845] "t"(<no params>) Ident [846-847] "a"

                    Stmt [850-865]
                        StmtKind: GateCall [850-865]: Ident [861-862] "s"
                        GateOperand IndexedIdent [863-864]: Ident [863-864] "a"[]
                Stmt [890-926]
                    StmtKind: Gate [890-926]: Ident [895-898] "tdg"(<no params>) Ident [899-900] "a"

                    Stmt [903-924]
                        StmtKind: GateCall [903-924]: Ident [920-921] "s"
                        GateOperand IndexedIdent [922-923]: Ident [922-923] "a"[]
                Stmt [946-975]
                    StmtKind: Gate [946-975]: Ident [951-953] "sx"(<no params>) Ident [954-955] "a"

                    Stmt [958-973]
                        StmtKind: GateCall [958-973]: Ident [969-970] "x"
                        GateOperand IndexedIdent [971-972]: Ident [971-972] "a"[]
                Stmt [1003-1056]
                    StmtKind: Gate [1003-1056]: Ident [1008-1010] "rx"(Ident [1011-1013] "θ") Ident [1015-1016] "a"

                    Stmt [1019-1040]
                        StmtKind: GateCall [1019-1040]: Ident [1019-1020] "U"
                        Expr [1021-1023]: Ident [1021-1023] "θ"
                        Expr [1025-1030]: BinOp (Div):
                            Expr [1025-1028]: UnOp (Neg):
                                Expr [1026-1028]: Ident [1026-1028] "π"
                            Expr [1029-1030]: Lit: Int(2)
                        Expr [1032-1036]: BinOp (Div):
                            Expr [1032-1034]: Ident [1032-1034] "π"
                            Expr [1035-1036]: Lit: Int(2)
                        GateOperand IndexedIdent [1038-1039]: Ident [1038-1039] "a"[]
                    Stmt [1041-1055]
                        StmtKind: GPhase [1041-1055]:
                        Expr [1048-1053]: BinOp (Div):
                            Expr [1048-1051]: UnOp (Neg):
                                Expr [1049-1051]: Ident [1049-1051] "θ"
                            Expr [1052-1053]: Lit: Int(2)
                Stmt [1083-1129]
                    StmtKind: Gate [1083-1129]: Ident [1088-1090] "ry"(Ident [1091-1093] "θ") Ident [1095-1096] "a"

                    Stmt [1099-1113]
                        StmtKind: GateCall [1099-1113]: Ident [1099-1100] "U"
                        Expr [1101-1103]: Ident [1101-1103] "θ"
                        Expr [1105-1106]: Lit: Int(0)
                        Expr [1108-1109]: Lit: Int(0)
                        GateOperand IndexedIdent [1111-1112]: Ident [1111-1112] "a"[]
                    Stmt [1114-1128]
                        StmtKind: GPhase [1114-1128]:
                        Expr [1121-1126]: BinOp (Div):
                            Expr [1121-1124]: UnOp (Neg):
                                Expr [1122-1124]: Ident [1122-1124] "θ"
                            Expr [1125-1126]: Lit: Int(2)
                Stmt [1156-1203]
                    StmtKind: Gate [1156-1203]: Ident [1161-1163] "rz"(Ident [1164-1166] "λ") Ident [1168-1169] "a"

                    Stmt [1172-1186]
                        StmtKind: GPhase [1172-1186]:
                        Expr [1179-1184]: BinOp (Div):
                            Expr [1179-1182]: UnOp (Neg):
                                Expr [1180-1182]: Ident [1180-1182] "λ"
                            Expr [1183-1184]: Lit: Int(2)
                    Stmt [1187-1201]
                        StmtKind: GateCall [1187-1201]: Ident [1187-1188] "U"
                        Expr [1189-1190]: Lit: Int(0)
                        Expr [1192-1193]: Lit: Int(0)
                        Expr [1195-1197]: Ident [1195-1197] "λ"
                        GateOperand IndexedIdent [1199-1200]: Ident [1199-1200] "a"[]
                Stmt [1223-1254]
                    StmtKind: Gate [1223-1254]: Ident [1228-1230] "cx"(<no params>) Ident [1231-1232] "a", Ident [1234-1235] "b"

                    Stmt [1238-1252]
                        StmtKind: GateCall [1238-1252]: Ident [1245-1246] "x"
                        GateOperand IndexedIdent [1247-1248]: Ident [1247-1248] "a"[]
                        GateOperand IndexedIdent [1250-1251]: Ident [1250-1251] "b"[]
                Stmt [1271-1302]
                    StmtKind: Gate [1271-1302]: Ident [1276-1278] "cy"(<no params>) Ident [1279-1280] "a", Ident [1282-1283] "b"

                    Stmt [1286-1300]
                        StmtKind: GateCall [1286-1300]: Ident [1293-1294] "y"
                        GateOperand IndexedIdent [1295-1296]: Ident [1295-1296] "a"[]
                        GateOperand IndexedIdent [1298-1299]: Ident [1298-1299] "b"[]
                Stmt [1319-1350]
                    StmtKind: Gate [1319-1350]: Ident [1324-1326] "cz"(<no params>) Ident [1327-1328] "a", Ident [1330-1331] "b"

                    Stmt [1334-1348]
                        StmtKind: GateCall [1334-1348]: Ident [1341-1342] "z"
                        GateOperand IndexedIdent [1343-1344]: Ident [1343-1344] "a"[]
                        GateOperand IndexedIdent [1346-1347]: Ident [1346-1347] "b"[]
                Stmt [1371-1410]
                    StmtKind: Gate [1371-1410]: Ident [1376-1378] "cp"(Ident [1379-1381] "λ") Ident [1383-1384] "a", Ident [1386-1387] "b"

                    Stmt [1390-1408]
                        StmtKind: GateCall [1390-1408]: Ident [1397-1398] "p"
                        Expr [1399-1401]: Ident [1399-1401] "λ"
                        GateOperand IndexedIdent [1403-1404]: Ident [1403-1404] "a"[]
                        GateOperand IndexedIdent [1406-1407]: Ident [1406-1407] "b"[]
                Stmt [1428-1469]
                    StmtKind: Gate [1428-1469]: Ident [1433-1436] "crx"(Ident [1437-1439] "θ") Ident [1441-1442] "a", Ident [1444-1445] "b"

                    Stmt [1448-1467]
                        StmtKind: GateCall [1448-1467]: Ident [1455-1457] "rx"
                        Expr [1458-1460]: Ident [1458-1460] "θ"
                        GateOperand IndexedIdent [1462-1463]: Ident [1462-1463] "a"[]
                        GateOperand IndexedIdent [1465-1466]: Ident [1465-1466] "b"[]
                Stmt [1487-1528]
                    StmtKind: Gate [1487-1528]: Ident [1492-1495] "cry"(Ident [1496-1498] "θ") Ident [1500-1501] "a", Ident [1503-1504] "b"

                    Stmt [1507-1526]
                        StmtKind: GateCall [1507-1526]: Ident [1514-1516] "ry"
                        Expr [1517-1519]: Ident [1517-1519] "θ"
                        GateOperand IndexedIdent [1521-1522]: Ident [1521-1522] "a"[]
                        GateOperand IndexedIdent [1524-1525]: Ident [1524-1525] "b"[]
                Stmt [1546-1587]
                    StmtKind: Gate [1546-1587]: Ident [1551-1554] "crz"(Ident [1555-1557] "θ") Ident [1559-1560] "a", Ident [1562-1563] "b"

                    Stmt [1566-1585]
                        StmtKind: GateCall [1566-1585]: Ident [1573-1575] "rz"
                        Expr [1576-1578]: Ident [1576-1578] "θ"
                        GateOperand IndexedIdent [1580-1581]: Ident [1580-1581] "a"[]
                        GateOperand IndexedIdent [1583-1584]: Ident [1583-1584] "b"[]
                Stmt [1604-1635]
                    StmtKind: Gate [1604-1635]: Ident [1609-1611] "ch"(<no params>) Ident [1612-1613] "a", Ident [1615-1616] "b"

                    Stmt [1619-1633]
                        StmtKind: GateCall [1619-1633]: Ident [1626-1627] "h"
                        GateOperand IndexedIdent [1628-1629]: Ident [1628-1629] "a"[]
                        GateOperand IndexedIdent [1631-1632]: Ident [1631-1632] "b"[]
                Stmt [1645-1690]
                    StmtKind: Gate [1645-1690]: Ident [1650-1654] "swap"(<no params>) Ident [1655-1656] "a", Ident [1658-1659] "b"

                    Stmt [1662-1670]
                        StmtKind: GateCall [1662-1670]: Ident [1662-1664] "cx"
                        GateOperand IndexedIdent [1665-1666]: Ident [1665-1666] "a"[]
                        GateOperand IndexedIdent [1668-1669]: Ident [1668-1669] "b"[]
                    Stmt [1671-1679]
                        StmtKind: GateCall [1671-1679]: Ident [1671-1673] "cx"
                        GateOperand IndexedIdent [1674-1675]: Ident [1674-1675] "b"[]
                        GateOperand IndexedIdent [1677-1678]: Ident [1677-1678] "a"[]
                    Stmt [1680-1688]
                        StmtKind: GateCall [1680-1688]: Ident [1680-1682] "cx"
                        GateOperand IndexedIdent [1683-1684]: Ident [1683-1684] "a"[]
                        GateOperand IndexedIdent [1686-1687]: Ident [1686-1687] "b"[]
                Stmt [1703-1748]
                    StmtKind: Gate [1703-1748]: Ident [1708-1711] "ccx"(<no params>) Ident [1712-1713] "a", Ident [1715-1716] "b", Ident [1718-1719] "c"

                    Stmt [1722-1746]
                        StmtKind: GateCall [1722-1746]: Ident [1736-1737] "x"
                        GateOperand IndexedIdent [1738-1739]: Ident [1738-1739] "a"[]
                        GateOperand IndexedIdent [1741-1742]: Ident [1741-1742] "b"[]
                        GateOperand IndexedIdent [1744-1745]: Ident [1744-1745] "c"[]
                Stmt [1768-1811]
                    StmtKind: Gate [1768-1811]: Ident [1773-1778] "cswap"(<no params>) Ident [1779-1780] "a", Ident [1782-1783] "b", Ident [1785-1786] "c"

                    Stmt [1789-1809]
                        StmtKind: GateCall [1789-1809]: Ident [1796-1800] "swap"
                        GateOperand IndexedIdent [1801-1802]: Ident [1801-1802] "a"[]
                        GateOperand IndexedIdent [1804-1805]: Ident [1804-1805] "b"[]
                        GateOperand IndexedIdent [1807-1808]: Ident [1807-1808] "c"[]
                Stmt [1872-1945]
                    StmtKind: Gate [1872-1945]: Ident [1877-1879] "cu"(Ident [1880-1882] "θ", Ident [1884-1886] "φ", Ident [1888-1890] "λ", Ident [1892-1894] "γ") Ident [1896-1897] "a", Ident [1899-1900] "b"

                    Stmt [1903-1916]
                        StmtKind: GateCall [1903-1916]: Ident [1903-1904] "p"
                        Expr [1905-1912]: BinOp (Sub):
                            Expr [1905-1907]: Ident [1905-1907] "γ"
                            Expr [1908-1912]: BinOp (Div):
                                Expr [1908-1910]: Ident [1908-1910] "θ"
                                Expr [1911-1912]: Lit: Int(2)
                        GateOperand IndexedIdent [1914-1915]: Ident [1914-1915] "a"[]
                    Stmt [1917-1943]
                        StmtKind: GateCall [1917-1943]: Ident [1924-1925] "U"
                        Expr [1926-1928]: Ident [1926-1928] "θ"
                        Expr [1930-1932]: Ident [1930-1932] "φ"
                        Expr [1934-1936]: Ident [1934-1936] "λ"
                        GateOperand IndexedIdent [1938-1939]: Ident [1938-1939] "a"[]
                        GateOperand IndexedIdent [1941-1942]: Ident [1941-1942] "b"[]
                Stmt [2003-2045]
                    StmtKind: Gate [2003-2045]: Ident [2008-2010] "CX"(<no params>) Ident [2011-2012] "a", Ident [2014-2015] "b"

                    Stmt [2018-2043]
                        StmtKind: GateCall [2018-2043]: Ident [2025-2026] "U"
                        Expr [2027-2029]: Ident [2027-2029] "π"
                        Expr [2031-2032]: Lit: Int(0)
                        Expr [2034-2036]: Ident [2034-2036] "π"
                        GateOperand IndexedIdent [2038-2039]: Ident [2038-2039] "a"[]
                        GateOperand IndexedIdent [2041-2042]: Ident [2041-2042] "b"[]
                Stmt [2060-2095]
                    StmtKind: Gate [2060-2095]: Ident [2065-2070] "phase"(Ident [2071-2073] "λ") Ident [2075-2076] "q"

                    Stmt [2079-2093]
                        StmtKind: GateCall [2079-2093]: Ident [2079-2080] "U"
                        Expr [2081-2082]: Lit: Int(0)
                        Expr [2084-2085]: Lit: Int(0)
                        Expr [2087-2089]: Ident [2087-2089] "λ"
                        GateOperand IndexedIdent [2091-2092]: Ident [2091-2092] "q"[]
                Stmt [2116-2163]
                    StmtKind: Gate [2116-2163]: Ident [2121-2127] "cphase"(Ident [2128-2130] "λ") Ident [2132-2133] "a", Ident [2135-2136] "b"

                    Stmt [2139-2161]
                        StmtKind: GateCall [2139-2161]: Ident [2146-2151] "phase"
                        Expr [2152-2154]: Ident [2152-2154] "λ"
                        GateOperand IndexedIdent [2156-2157]: Ident [2156-2157] "a"[]
                        GateOperand IndexedIdent [2159-2160]: Ident [2159-2160] "b"[]
                Stmt [2189-2216]
                    StmtKind: Gate [2189-2216]: Ident [2194-2196] "id"(<no params>) Ident [2197-2198] "a"

                    Stmt [2201-2214]
                        StmtKind: GateCall [2201-2214]: Ident [2201-2202] "U"
                        Expr [2203-2204]: Lit: Int(0)
                        Expr [2206-2207]: Lit: Int(0)
                        Expr [2209-2210]: Lit: Int(0)
                        GateOperand IndexedIdent [2212-2213]: Ident [2212-2213] "a"[]
                Stmt [2249-2281]
                    StmtKind: Gate [2249-2281]: Ident [2254-2256] "u1"(Ident [2257-2259] "λ") Ident [2261-2262] "q"

                    Stmt [2265-2279]
                        StmtKind: GateCall [2265-2279]: Ident [2265-2266] "U"
                        Expr [2267-2268]: Lit: Int(0)
                        Expr [2270-2271]: Lit: Int(0)
                        Expr [2273-2275]: Ident [2273-2275] "λ"
                        GateOperand IndexedIdent [2277-2278]: Ident [2277-2278] "q"[]
                Stmt [2282-2347]
                    StmtKind: Gate [2282-2347]: Ident [2287-2289] "u2"(Ident [2290-2292] "φ", Ident [2294-2296] "λ") Ident [2298-2299] "q"

                    Stmt [2302-2326]
                        StmtKind: GPhase [2302-2326]:
                        Expr [2309-2324]: BinOp (Div):
                            Expr [2309-2322]: UnOp (Neg):
                                Expr [2310-2322]: Paren:
                                    Expr [2311-2321]: BinOp (Add):
                                        Expr [2311-2316]: BinOp (Add):
                                            Expr [2311-2313]: Ident [2311-2313] "φ"
                                            Expr [2314-2316]: Ident [2314-2316] "λ"
                                        Expr [2317-2321]: BinOp (Div):
                                            Expr [2317-2319]: Ident [2317-2319] "π"
                                            Expr [2320-2321]: Lit: Int(2)
                            Expr [2323-2324]: Lit: Int(2)
                    Stmt [2327-2345]
                        StmtKind: GateCall [2327-2345]: Ident [2327-2328] "U"
                        Expr [2329-2333]: BinOp (Div):
                            Expr [2329-2331]: Ident [2329-2331] "π"
                            Expr [2332-2333]: Lit: Int(2)
                        Expr [2335-2337]: Ident [2335-2337] "φ"
                        Expr [2339-2341]: Ident [2339-2341] "λ"
                        GateOperand IndexedIdent [2343-2344]: Ident [2343-2344] "q"[]
                Stmt [2348-2413]
                    StmtKind: Gate [2348-2413]: Ident [2353-2355] "u3"(Ident [2356-2358] "θ", Ident [2360-2362] "φ", Ident [2364-2366] "λ") Ident [2368-2369] "q"

                    Stmt [2372-2394]
                        StmtKind: GPhase [2372-2394]:
                        Expr [2379-2392]: BinOp (Div):
                            Expr [2379-2390]: UnOp (Neg):
                                Expr [2380-2390]: Paren:
                                    Expr [2381-2389]: BinOp (Add):
                                        Expr [2381-2386]: BinOp (Add):
                                            Expr [2381-2383]: Ident [2381-2383] "φ"
                                            Expr [2384-2386]: Ident [2384-2386] "λ"
                                        Expr [2387-2389]: Ident [2387-2389] "θ"
                            Expr [2391-2392]: Lit: Int(2)
                    Stmt [2395-2411]
                        StmtKind: GateCall [2395-2411]: Ident [2395-2396] "U"
                        Expr [2397-2399]: Ident [2397-2399] "θ"
                        Expr [2401-2403]: Ident [2401-2403] "φ"
                        Expr [2405-2407]: Ident [2405-2407] "λ"
                        GateOperand IndexedIdent [2409-2410]: Ident [2409-2410] "q"[]"#]],
    );
}
