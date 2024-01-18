# Resource Estimation for Double-factorized Chemistry

In this sample we evaluate the physical resource estimates of using the so-called double-factorized qubitization algorithm described in [[Phys. Rev. Research 3, 033055 (2021)](https://doi.org/10.1103/PhysRevResearch.3.033055)] to calculate the energy of a user provided Hamiltonian to chemical accuracy of 1 mHa. 

The Hamiltonian is provided as an FCIDUMP file that is available on your machine or can be downloaded via an HTTPS URL.

```
usage: chemistry.py [-h] [-f FCIDUMPFILE]

Double-factorized chemistry sample

options:
  -h, --help            show this help message and exit
  -f FCIDUMPFILE, --fcidumpfile FCIDUMPFILE
                        Path to the FCIDUMP file describing the Hamiltonian
```

For example, the following command will download the FCIDUMP file `n2-10e-8o` to the working folder and run resource estimation for it:

```
chemistry.py -f https://aka.ms/fcidump/n2-10e-8o
```

After that, you can pass the path to the downloaded file to the script instead:

```
chemistry.py -f n2-10e-8o
```

You can choose some of the following URLs to download example files:

| URL  | Instance name  | Description  |
|------|----------------|--------------|
| https://aka.ms/fcidump/n2-10e-8o             | n2-10e-8o             | 10 electron, 8 orbital active space of he dissociated nitrogen at 3 Angstrom distance (the default file used in the example) |
| https://aka.ms/fcidump/XVIII-cas4-fb-64e-56o | XVIII-cas4-fb-64e-56o | 64 electron, 56 orbital active space of one of the stable intermediates in the [ruthenium-catalyzed carbon fixation cycle](https://journals.aps.org/prresearch/abstract/10.1103/PhysRevResearch.3.033055) |
| https://aka.ms/fcidump/nitrogenase-54e-54o   | nitrogenase-54e-54o   | 54 electron, 54 orbital active space of the active core of the nitrogenase that is used in [this paper](https://www.pnas.org/doi/10.1073/pnas.1619152114) |
| https://aka.ms/fcidump/fe2s2-10e-40o         | fe2s2-10e-40o         | 10 electron, 40 orbital active space of [2Fe, 2S] cluster that is shown in [this paper](https://www.nature.com/articles/nchem.2041) |
| https://aka.ms/fcidump/polyyne-24e-24o       | polyyne-24e-24o       | 24 electron, 24 orbital active space of the polyyne molecule |

The numbers for the XVIII-cas4-fb-64e-56o instance roughly match the numbers in the paper [Assessing requirements for scaling quantum computers to real-world impact](https://aka.ms/AQ/RE/Paper), as we incorporated a few improvements in the implementation of the double-factorized chemistry algorithm as compared to the version used when the paper was published.