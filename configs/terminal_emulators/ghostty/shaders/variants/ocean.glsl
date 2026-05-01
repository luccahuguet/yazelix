// Ocean blue variant

const vec4 TRAIL_COLOR = vec4(0.37, 0.66, 1.00, 1.0);    // ~#5FA8FF
const vec4 TRAIL_COLOR_ACCENT = vec4(0.12, 0.40, 0.96, 1.0); // ~#1E66F5
const float DURATION = 0.26;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderMonoColorTrail(fragColor, fragCoord, TRAIL_COLOR, TRAIL_COLOR_ACCENT, DURATION, .006, 1.4);
}
