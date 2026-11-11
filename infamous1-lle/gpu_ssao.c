/*
 * gpu_ssao.c -- inFAMOUS 1 placeholder.
 *
 * The inFAMOUS 2 port ran the SSAO job's five passes on the GPU
 * (PS3RECOMP_ENG_GPU_SSAO=<model target>). inFAMOUS 1's SSAO job was never
 * mapped, so this build carries no GPU SSAO: the job runs on the SPUs.
 */

void infamous1_gpu_ssao_install(void)
{
    /* nothing yet: PS3RECOMP_ENG_GPU_SSAO is not a recognised knob in this
     * port (run.sh leaves it unset) */
}
