# fake-space

`fake-space` is a game. Check out [`pixels`](https://github.com/parasyte/pixels) which helped me learn more about this type of rendering.

Excellent explanations about the raycaster [here](https://lodev.org/cgtutor/raycasting.html#The_Basic_Idea_).

## Roadmap

- [x] Implement a working 2D raycaster with a specific FOV 
- [x] Draw walls out of distance values
- [x] Add textures to walls
- [x] Add special textures with half transparency or full transparency
- [x] Add textures to floor and ceiling
- [x] Add voxel objects
- [x] Add different height walls
- [x] Add colors to voxel objects
- [x] Render objects from closest to furthest while skipping drawing over already drawn full opacity pixels in order to increase performance
- [x] Implement drawing directly in the ray casting function to avoid mem allocations, to work easier with entities and increase performance
- [x] Add different floor and ceiling textures for different map tiles
- [ ] Add an ability to look freely up and down, jump and crouch
- [ ] Add 2D sprites (i.e. pillars, barrels)
- [ ] Add removable object walls (cool opening or moving animation with voxels)
- [ ] Add collision detection (circle-rectangle collision detection)
- [ ] Draw a skybox when outside of the map (looking at out of bound parts or at transparent ceiling)
- [ ] Add portals and a portal gun like in the Portal game
- [ ] Add special tiles through which the player would fall to their demise
- [ ] Add an UI
- [ ] Rename `hits` to `casts` and store transparent wall hits, object hits and normal wall hits in a single Vec
- [ ] Add an option to switch between CPU only graphics with lower quality (no 3D objects) and GPU accelerated graphics with higher quality (3D objects)
- [ ] Obsolete the need for the `image` crate for minimal size and faster compile times
- [ ] Go unsafe for performance increase after most of the project is finished