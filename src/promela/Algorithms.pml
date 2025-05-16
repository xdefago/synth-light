#ifndef __ALGORITHMS_PML__
#define __ALGORITHMS_PML__
#  define ALGO_NAME      "ALGO_SYNTH_00s_01s_10s_11s_00d_01d_10d_11d__S0_S0_S1_S1_S1_S0_O1_H0"
#  define Algorithm(o,c) Alg_Synth(o,c)
#  define MAX_COLOR      (2)
#  define NUM_COLORS     (2)
inline Alg_Synth(obs, command)
{
    command.move      = STAY;
    command.new_color = obs.color.me;
    if
    :: (obs.color.me == 0) && (obs.color.other == 0) && (obs.same_position) -> command.move = STAY; command.new_color = 0;
    :: (obs.color.me == 0) && (obs.color.other == 1) && (obs.same_position) -> command.move = STAY; command.new_color = 0;
    :: (obs.color.me == 1) && (obs.color.other == 0) && (obs.same_position) -> command.move = STAY; command.new_color = 1;
    :: (obs.color.me == 1) && (obs.color.other == 1) && (obs.same_position) -> command.move = STAY; command.new_color = 1;
    :: (obs.color.me == 0) && (obs.color.other == 0) && ! (obs.same_position) -> command.move = STAY; command.new_color = 1;
    :: (obs.color.me == 0) && (obs.color.other == 1) && ! (obs.same_position) -> command.move = STAY; command.new_color = 0;
    :: (obs.color.me == 1) && (obs.color.other == 0) && ! (obs.same_position) -> command.move = TO_OTHER; command.new_color = 1;
    :: (obs.color.me == 1) && (obs.color.other == 1) && ! (obs.same_position) -> command.move = TO_HALF; command.new_color = 0;
    fi;
}
#endif
