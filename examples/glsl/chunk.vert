#version 130

uniform ivec3 chunk_xyz;
uniform mat4 view;
uniform mat4 projection;

uniform vec3 eyePosXYZ;
 
layout(location=0) vec3 vertXYZ; 
layout(location=1) vec2 vertUV; 
layout(location=2) vec3 vertNormalXYZ;
smooth out vec3 fragXYZ;
smooth out vec2 fragUV;
smooth out vec3 fragNormalXYZ;


// todo: clearly needs to be optimized.
vec2 clamp_box(vec2 c) 
{
	// south side
	if (c.x > c.z && -c.x > c.z)
	{
		return vec2(c.x / (-2.f * c.z), -0.5f);
	}
	// north side
	else if (c.x <= c.z && -c.x <= c.z)
	{
		return vec2(c.x / (2.f * c.z), 0.5f);
	}
	// east side
	else if (c.x > c.z && -c.x <= c.z)
	{
		return vec2(0.5f, c.z / (2.f * c.x))
	}
	// west side
	else if (c.x > c.z && -c.x <= c.z)
	{
		return vec2(-0.5f, c.z / (-2.f * c.x))
	}
}

void main()
{
	// ENSURE_9182327826
	// REQUIRE_239487367843 {
	//     if is_billbard then frac(vertXYZ.y) is 0.5f
	// }
	bool is_billboard = (vertNormalXYZ.x + vertNormalXYZ.y + vertNormal.XYZ.z) < .1f;
	vec3 billboard_normal = vertXYZ - (eyePosXYZ - chunk_xyz);
	billboard_normal.z = 0;
	float billboard_x_side = (frac(vertXYZ.x) - .5f) / .3f;
	vertXYZ.x -= billboard_x_side * .3f; // set frac(vertXYZ.x) to .5f
	vec3 x_correction = billboard_x_side * cross(billboard_normal, vec3(0, 1, 0));
	x_correction.xz = clamp_box(x_correction.xz);
	x_correction*= is_billboard;
	billboard_normal*= is_billboard;

	gl_Position = projection * view * model * vec4(vertXYZ + x_correction, 1);
	fragXYZ = vertXYZ;
	fragUV = vertUV;
	fragNormalXYZ = vertNormalXYZ + billboard_normal;
}

