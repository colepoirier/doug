- implement a window displaying information about the currently selected shape (net name, layer, entity id, dimensions, position)
- think about if a window dipslaying information about multiple currently selected shapes makes sense and how that information would be displayed
- implement undo/redo functionality with file menu and keyboard shortcuts
- implement a physical unit grid and snap-to functionality for editing
- implement file saving to gds and proto and possibly a custom fast loading file format using rkyv
- implement selected shapes 'corner and midpoints' to indicate where the shape can be clicked-and-dragged to edit from, add associated editing functionality.
- Add ability to delete shapes (needs to be tracked in undo/redo)
- Add functionality to draw new shapes (rects should be easy, paths and polygons will require a lot more thought)

- add ability to change layer colors? (should rethink Layer colors and coloring in general guided by 'Selecting Colors for Representing VLSI Layout by Giordano Bruno Beretta - Xerox Paulo Alto Research Center(1988)')

- use bevy_data size to cap max memory that can be used by a cell's bevy_protoype_lyon shapes
- use cxx to create FFI safe bindings to tiledb

- use rkyv to back the editable cell data, and create a way to serialize this back to the proto/gds file that the un-edited cell was loaded from
- render LODs of qoi images from the svg shapes for large designs, store them in tiledb, and implement data storage in tile db for each tile of each LOD level so that zooming in and out works smoothly, as do mass edits such as chaning a layer color, track width... other operations... as well as querying by net name and highliting those shapes in the zoomed out view so one can identify HDL modules in the physical layout
- implement unit and integration tests
