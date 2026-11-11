/*
 * gpu_light.c -- inFAMOUS 1 placeholder.
 *
 * The inFAMOUS 2 port ran this engine's light job as a Metal pass
 * (PS3RECOMP_ENG_GPU_LIGHT=<job sha>:<controller>). inFAMOUS 1's light job
 * module was never mapped, so this build carries no GPU light pass: the game
 * runs the job on the SPUs, exactly as shipped.
 */

void infamous1_gpu_light_install(void)
{
    /* nothing yet: PS3RECOMP_ENG_GPU_LIGHT is not a recognised knob in this
     * port (run.sh leaves it unset) */
}
