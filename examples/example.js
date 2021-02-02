// vertexshader.glsl:
// in vec2 xy;
// in vec2 uv;

// calling gl.vertexAttribPointer sucks, it couldnt be more redundant, its easy to forget one thing and break everything.
// even this helper function i wrote requires you to specify the stride and offset, which is as well redundant (->introduces errors)
// and looks ugly

let sizeof_float = 4;
let the_vao = ...;
let the_vbo = ...;

gl.h.vertexAttribute(the_vao, { 
    attrib_name: "pos", 
    component_count: 2, 
    component_type: gl.FLOAT,
    stride: (2 + 2) * sizeof_float, 
    offset: 0,
    source_buffer: the_vbo
});

gl.h.vertexAttribute(the_vao, { 
    attrib_name: "uv", 
    component_count: 2, 
    component_type: gl.FLOAT,
    stride: (2 + 2) * sizeof_float, 
    offset: 2 * sizeof_float,
    source_buffer: the_vbo
});

// so quickly write up some macros...

/*
    








































































































































































































    












       




*/

// e voila, now we have this much more concise and nice-looking syntax :)


    gl.h.vertexAttribute(the_vao, {
attrib_name: "pos",
component_count: 2,
component_type: gl.FLOAT,
stride: 16,
offset: 0,
source_buffer: the_vbo
});
gl.h.vertexAttribute(the_vao, {
attrib_name: "uv",
component_count: 2,
component_type: gl.FLOAT,
stride: 16,
offset: 8,
source_buffer: the_vbo
});
 
