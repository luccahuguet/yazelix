// Blaze fire variant

const vec4 TRAIL_COLOR = vec4(1.0, 0.725, 0.161, 1.0);
const vec4 TRAIL_COLOR_ACCENT = vec4(1.0, 0.0, 0.0, 1.0);
const float DURATION = 0.3;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderMonoColorTrail(fragColor, fragCoord, TRAIL_COLOR, TRAIL_COLOR_ACCENT, DURATION, .007, 1.5);
}
