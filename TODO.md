- implement click-and-drag rectangular area selection of shapes
- implement click on shape and drag to move (may be tricky to get the UX on this right)
- implement selected shapes 'corner and midpoints' to indicate where the shape can be clicked-and-dragged to edit from, add associated editing functionality

- add ability to change layer colors? (should rethink Layer colors and coloring in general guided by 'Selecting Colors for Representing VLSI Layout by Giordano Bruno Beretta - Xerox Paulo Alto Research Center(1988)')

- use bevy_data size to cap max memory that can be used by a cell's bevy_protoype_lyon shapes
- use cxx to create FFI safe bindings to tiledb

- use rkyv to back the editable cell data, and create a way to serialize this back to the proto/gds file that the un-edited cell was loaded from
- render LODs of qoi images from the svg shapes for large designs, store them in tiledb, and implement data storage in tile db for each tile of each LOD level so that zooming in and out works smoothly, as do mass edits such as chaning a layer color, track width... other operations... as well as querying by net name and highliting those shapes in the zoomed out view so one can identify HDL modules in the physical layout
- implement unit and integration tests
