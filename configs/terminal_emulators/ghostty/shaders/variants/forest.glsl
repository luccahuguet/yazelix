// Forest green variant

const vec4 TRAIL_COLOR = vec4(0.23, 0.82, 0.48, 1.0);      // ~#3AD07A
const vec4 TRAIL_COLOR_ACCENT = vec4(0.11, 0.62, 0.35, 1.0); // ~#1B9E59
const float DURATION = 0.25;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    renderSimpleDualColorTrail(fragColor, fragCoord, TRAIL_COLOR, TRAIL_COLOR_ACCENT, DURATION, .006, 1.4);
}
