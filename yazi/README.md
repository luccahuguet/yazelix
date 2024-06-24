# Yazellix v2: Yazi Files

### Overview

Yazellix v2 integrates yazi, zellij and helix in a smooth experience.
- Zellij manages everything, with yazi as a sidebar and helix as the editor
- And helix is called when you select a file in the "sidebar", opening as a new pane in zellij
- You can open and close the sidebar by switching zellij layouts (press `alt ]` and `alt [`)

 
## Screenshot

![image](https://github.com/luccahuguet/yazi-files/assets/27565287/557eecbf-6eeb-48f9-8de4-252f78bda4fd)


### Details
- Ratio is set to [0, 4, 0], so 0/4 width for parent, 4/4 width for current, 0/4 width for preview
- This ratio was chosen because in my setup yazi only has 20% of the total width
- There is now a init.lua file that makes the status bar way more readable (but you don't need lua as a dependency, so no worries)
- The init.lua file was added as a PR by the yazi creator themselves! What an honor, honestly.


## More info

Check out the complete setup at my [zellij files repo](https://github.com/luccahuguet/zellij-files)
