## Fragments

<table id="fragments">
  <thead>
    <tr>
      <th colspan="2">ID</th>
      <th colspan="3">Name</th>
    </tr>
    <tr>
      <th>Hex</th>
      <th>Dec</th>
      <th>libeq</th>
      <th>Windcatcher</th>
      <th>WLDCOM.EXE</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>0x01</td>
      <td>1</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/default_palette_file.rs">DefaultPaletteFile</a></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#defaultpalettefile---0x01">DEFAULTPALETTEFILE %s</a></td>
    </tr>
    <tr>
      <td>0x02</td>
      <td>2</td>
      <td></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#userdata---0x02">USERDATA %s</a></td>
    </tr>
    <tr>
      <td>0x03</td>
      <td>3</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/bm_info.rs">BmInfo</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x03--texture-bitmap-names--plain">Texture Bitmap Name(s)</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#frame---0x03">FRAME and BMINFO</a></td>
    </tr>
    <tr>
      <td>0x04</td>
      <td>4</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/simple_sprite_def.rs">SimpleSpriteDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x04--texture-bitmap-info--plain">Texture Bitmap Info</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#simplespritedef---0x04">SIMPLESPRITEDEF</a></td>
    </tr>
    <tr>
      <td>0x05</td>
      <td>5</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/simple_sprite.rs">SimpleSprite</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x05--texture-bitmap-info-reference--reference">Texture Bitmap Info Reference</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#simplespriteinst---0x05">SIMPLESPRITEINST</a></td>
    </tr>
    <tr>
      <td>0x06</td>
      <td>6</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/sprite_2d_def.rs">Sprite2DDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x06--two-dimensional-object--plain">Two-Dimensional Object</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#2dspritedef---0x06">2DSPRITEDEF</a></td>
    </tr>
    <tr>
      <td>0x07</td>
      <td>7</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/sprite_2d.rs">Sprite2D</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x07--camera-reference--reference">Two-Dimensional Object Reference</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0x07">2DSPRITE (ref)</a></td>
    </tr>
    <tr>
      <td>0x08</td>
      <td>8</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/sprite_3d_def.rs">Sprite3DDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x08--camera--plain">Camera</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#3dspritedef---0x08">3DSPRITEDEF</a></td>
    </tr>
    <tr>
      <td>0x09</td>
      <td>9</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/sprite_3d.rs">Sprite3D</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x09--camera-reference--reference">Camera Reference</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0x09">3DSPRITE (ref)</a></td>
    </tr>
    <tr>
      <td>0x0a</td>
      <td>10</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/sprite_4d_def.rs">Sprite4DDef</a></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#4dspritedef---0xa">4DSPRITEDEF</a></td>
    </tr>
    <tr>
      <td>0x0b</td>
      <td>11</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/sprite_4d.rs">Sprite4D</a></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0xb">4DSPRITE (ref)</a></td>
    </tr>
    <tr>
      <td>0x0c</td>
      <td>12</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/particle_sprite_def.rs">ParticleSpriteDef</a></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#particlespritedef---0xc">PARTICLESPRITEDEF</a></td>
    </tr>
    <tr>
      <td>0x0d</td>
      <td>13</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/particle_sprite.rs">ParticleSprite</a></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0xd">PARTICLESPRITE (ref)</a></td>
    </tr>
    <tr>
      <td>0x0e</td>
      <td>14</td>
      <td></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#compositespritedef---0xe">COMPOSITESPRITEDEF</a></td>
    </tr>
    <tr>
      <td>0x0f</td>
      <td>15</td>
      <td></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0xf">COMPOSITESPRITE (ref)</a></td>
    </tr>
    <tr>
      <td>0x10</td>
      <td>16</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/hierarchical_sprite_def.rs">HierarchicalSpriteDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x10--skeleton-track-set--plain">Skeleton Track Set</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#hierarchicalspritedef---0x10">HIERARCHICALSPRITEDEF</a></td>
    </tr>
    <tr>
      <td>0x11</td>
      <td>17</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/hierarchical_sprite.rs">HierarchicalSprite</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x11--skeleton-track-set-reference--reference">Skeleton Track Set Reference</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0x11">HIERARCHICALSPRITE (ref)</a></td>
    </tr>
    <tr>
      <td>0x12</td>
      <td>18</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/track_def.rs">TrackDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x12--mob-skeleton-piece-track--plain">Mob Skeleton Piece Track</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#trackdefinition--0x12">TRACKDEFINITION</a></td>
    </tr>
    <tr>
      <td>0x13</td>
      <td>19</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/track.rs">Track</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x13--mob-skeleton-piece-track-reference--reference">Mob Skeleton Piece Track Reference</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#trackinstance---0x13">TRACKINSTANCE</a></td>
    </tr>
    <tr>
      <td>0x14</td>
      <td>20</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/actor_def.rs">ActorDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x14--static-or-animated-model-referenceplayer-info--plain">Static or Animated Model Reference/Player Info</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#actordef---0x14">ACTORDEF</a></td>
    </tr>
    <tr>
      <td>0x15</td>
      <td>21</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/actor.rs">Actor</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x15--object-location--reference">Object Location</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#actorinst---0x15">ACTORINST</a></td>
    </tr>
    <tr>
      <td>0x16</td>
      <td>22</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/sphere.rs">Sphere</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x16--zone-unknown--plain">Zone Unknown</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0x16">SPHERE (ref)</a></td>
    </tr>
    <tr>
      <td>0x17</td>
      <td>23</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/polyhedron_def.rs">PolyhedronDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x17--polygon-animation--plain">Polygon Animation?</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#polyhedrondefinition---0x17">POLYHEDRONDEFINITION</a></td>
    </tr>
    <tr>
      <td>0x18</td>
      <td>24</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/polyhedron.rs">Polyhedron</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x18--polygon-animation-reference--reference">Polygon Animation Reference?</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0x18">POLYHEDRON (ref)</a></td>
    </tr>
    <tr>
      <td>0x19</td>
      <td>25</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/sphere_list_def.rs">SphereListDef</a></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#spherelistdefinition---0x19">SPHERELISTDEFINITION</a></td>
    </tr>
    <tr>
      <td>0x1a</td>
      <td>26</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/sphere_list.rs">SphereList</a></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0x1a">SPHERELIST (ref)</a></td>
    </tr>
    <tr>
      <td>0x1b</td>
      <td>27</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/light_def.rs">LightDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x1b--light-source--plain">Light Source</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#lightdefinition---0x1b">LIGHTDEFINITION</a></td>
    </tr>
    <tr>
      <td>0x1c</td>
      <td>28</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/light.rs">Light</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x1c--light-source-reference--reference">Light Source Reference</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0x1c">LIGHT (ref)</a></td>
    </tr>
    <tr>
      <td>0x1d</td>
      <td>29</td>
      <td></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#pointlight---0x1d">POINTLIGHT ????</a></td>
    </tr>
    <tr>
      <td>0x1e</td>
      <td>30</td>
      <td></td>
      <td></td>
      <td></td>
    </tr>
    <tr>
      <td>0x1f</td>
      <td>31</td>
      <td></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#sounddefinition---0x1f">SOUNDDEFINITION</a></td>
    </tr>
    <tr>
      <td>0x20</td>
      <td>32</td>
      <td></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#soundinstance---0x20">SOUNDINSTANCE</a></td>
    </tr>
    <tr>
      <td>0x21</td>
      <td>33</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/world_tree.rs">WorldTree</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x21--bsp-tree--plain">BSP Tree</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#worldtree---0x21">WORLDTREE</a></td>
    </tr>
    <tr>
      <td>0x22</td>
      <td>34</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/region.rs">Region</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x22--bsp-region--plain">Bsp Region</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#region---0x22">REGION</a></td>
    </tr>
    <tr>
      <td>0x23</td>
      <td>35</td>
      <td></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#activegeometryregion--0x23">ACTIVEGEOMETRYREGION</a></td>
    </tr>
    <tr>
      <td>0x24</td>
      <td>36</td>
      <td></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#skyregion---0x24">SKYREGION</a></td>
    </tr>
    <tr>
      <td>0x25</td>
      <td>37</td>
      <td></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#directionallight---0x25">DIRECTIONALLIGHT ????</a></td>
    </tr>
    <tr>
      <td>0x26</td>
      <td>38</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/blit_sprite_def.rs">BlitSpriteDef</a></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#blitspritedefinition---0x26">BLITSPRITEDEFINITION</a></td>
    </tr>
    <tr>
      <td>0x27</td>
      <td>39</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/blit_sprite.rs">BlitSprite</a></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#0x27">BLITSPRITE (ref)</a></td>
    </tr>
    <tr>
      <td>0x28</td>
      <td>40</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/point_light.rs">PointLight</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x28--light-info--reference">Light Info</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#pointlight-regions---0x28">POINTLIGHT</a></td>
    </tr>
    <tr>
      <td>0x29</td>
      <td>41</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/zone.rs">Zone</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x29--region-flag--plain">Region Flag</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#zone---0x29">ZONE</a></td>
    </tr>
    <tr>
      <td>0x2a</td>
      <td>42</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/ambient_light.rs">AmbientLight</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x2a--ambient-light--reference">Ambient Light</a></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#ambientlight---0x2a">AMBIENTLIGHT</a></td>
    </tr>
    <tr>
      <td>0x2b</td>
      <td>43</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/directional_light.rs">DirectionalLight</a></td>
      <td></td>
      <td><a href="https://wld-doc.github.io/object-types/overview/#directionallight-static-flag---0x2b">DIRECTIONALLIGHT</a></td>
    </tr>
    <tr>
      <td>0x2c</td>
      <td>44</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/dm_sprite_def.rs">DmSpriteDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x2c--alternate-mesh--plain">Alternate Mesh</a></td>
      <td>DMSPRITEDEF (suspected)</td>
    </tr>
    <tr>
      <td>0x2d</td>
      <td>45</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/dm_sprite.rs">DmSprite</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x2d--mesh-reference--reference">Mesh Reference</a></td>
      <td></td>
    </tr>
    <tr>
      <td>0x2e</td>
      <td>46</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/dm_track_def.rs">DmTrackDef</a></td>
      <td></td>
      <td></td>
    </tr>
    <tr>
      <td>0x2f</td>
      <td>47</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/dm_track.rs">DmTrack</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x2f--mesh-animated-vertices-reference--reference">Mesh Animated Vertices Reference</a></td>
      <td></td>
    </tr>
    <tr>
      <td>0x30</td>
      <td>48</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/material_def.rs">MaterialDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x30--texture--reference">Texture</a></td>
      <td>MATERIALDEFINITION</td>
    </tr>
    <tr>
      <td>0x31</td>
      <td>49</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/material_palette.rs">MaterialPalette</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x31--texture-list--plain">TextureList</a></td>
      <td>MATERIALPALETTE</td>
    </tr>
    <tr>
      <td>0x32</td>
      <td>50</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/dm_rgb_track_def.rs">DmRGBTrackDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x32--vertex-color--plain">Vertex Color</a></td>
      <td></td>
    </tr>
    <tr>
      <td>0x33</td>
      <td>51</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/dm_rgb_track.rs">DmRGBTrack</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x33--vertex-color-reference--reference">Vertex Color Reference</a></td>
      <td></td>
    </tr>
    <tr>
      <td>0x34</td>
      <td>52</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/particle_cloud_def.rs">ParticleCloudDef</a></td>
      <td></td>
      <td></td>
    </tr>
    <tr>
      <td>0x35</td>
      <td>53</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/global_ambient_light_def.rs">GlobalAmbientLightDef</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x35--first-fragment--plain">First Fragment</a></td>
      <td></td>
    </tr>
    <tr>
      <td>0x36</td>
      <td>54</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/dm_sprite_def_2.rs">DmSpriteDef2</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x36--mesh--plain">Mesh</a></td>
      <td>DMSPRITEDEF2</td>
    </tr>
    <tr>
      <td>0x37</td>
      <td>55</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/dm_track_def_2.rs">DmTrackDef2</a></td>
      <td><a href="https://github.com/EQEmu/eqemu-docs-v2/blob/main/docs/server/zones/customizing-zones/wld-file-reference.md#0x37--mesh-animated-vertices--plain">Mesh Animated Vertices</a></td>
      <td>DMTRACKDEF</td>
    </tr>
  </tbody>
</table>

## Other Fragments
<table id="other-fragments">
  <thead>
    <tr>
      <th colspan="3">ID</th>
      <th colspan="2">Name</th>
    </tr>
    <tr>
      <th>Hex</th>
      <th>Dec</th>
      <th>Game</th>
      <th>libeq</th>
      <th>WLDCOM.EXE</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>0x2c</td>
      <td>44</td>
      <td>Tanarus</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/world_vertices.rs">WorldVertices</a></td>
      <td>WORLDVERTICES</td>
    </tr>
    <tr>
      <td>0x2c</td>
      <td>44</td>
      <td>Return to Krondor</td>
      <td><a href="https://github.com/cjab/libeq/blob/master/crates/libeq_wld/src/parser/fragments/bm_info_rtk.rs">BmInfoRtk</a></td>
      <td>BMINFO</td>
    </tr>
  </tbody>
</table>

## ASCII File Extensions
<table id="ascii-file-extensions">
  <thead>
    <tr>
      <th>Extension</th>
      <th>Contents</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>MDF</td>
      <td>Material</td>
    </tr>
    <tr>
      <td>SPH</td>
      <td>Animation</td>
    </tr>
    <tr>
      <td>SPK</td>
      <td>Skin Exporter (Mesh, Hierarchical Sprite data)</td>
    </tr>
    <tr>
      <td>SPS</td>
      <td>Simple Sprite Def (Texture)</td>
    </tr>
  </tbody>
</table>
