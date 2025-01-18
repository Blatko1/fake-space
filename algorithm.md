
Algorithm:
1. Load all textures from the `textures` folder:
	- Load textures into a `HashMap<String, TextureData>` so they could be easily accessed when parsing `Blueprints`
	- Each texture has a unique `TextureID`
2. Load all blueprints from the `blueprints` folder:
	- Each blueprint is stored in it's own unique folder under the `blueprints` folder
	- Inside each unique folder find the `.tmx` file and load properties (ambient, skybox, ...) and the `TileLayer`
	- Each tile must have properties: 4 height levels, 4 texture names; and may have properties: portal direction
	- Specified tile texture names also must exist in the previously created texture `HashMap` so a valid `TextureID` could be stored 

Ideas:
- Rename: `TextureData` -> `Texture` 