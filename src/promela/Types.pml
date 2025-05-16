#ifndef __TYPES_PML__
#define __TYPES_PML__

#define RIGID       (100)
#define NON_RIGID   (101)


#define robot_t	bit
#define ROBOT_A	0
#define ROBOT_B	1
#define otherRobot(me)	((me) != ROBOT_A -> ROBOT_A : ROBOT_B)

#define color_t	byte
#define BLACK	0
#define	WHITE	1
/* Three-four color schemes */
#define RED		2
#define YELLOW	3
#define GREEN	4

#define position_t	mtype
mtype { SAME, NEAR, FAR };

#define move_t	mtype
mtype { STAY, TO_HALF, TO_OTHER, MISS };

#define GATHERED	(position == SAME && robot[ROBOT_A].pending == STAY && robot[ROBOT_B].pending == STAY)

typedef robot_state_external_t {
    color_t		color;
    bool		is_moving	 = false;
    bool		is_computing = false;
    color_t		pending_color;
    move_t		pending		 = STAY;
};

typedef color_tuple_t {
    color_t	me;
    color_t	other
};

typedef observation_t {
    color_tuple_t	color;
    bool			same_position;
    bool			near_position
};

typedef command_t {
    move_t	move;
    color_t	new_color
};

mtype { LOOK, BEGIN_COMPUTE, END_COMPUTE, BEGIN_MOVE, END_MOVE };

#define MAX_PHASES  (END_MOVE - LOOK + 1)
#define nextPhase(phase)   ( ((phase) > END_MOVE) -> ((phase)-1) : LOOK )

show position_t				position = FAR;
show robot_state_external_t	robot[2];

chan robot_in[2] = [0] of { mtype, chan };

#define FAIR_LIMIT(phasesPerCycle, numColors) (1 + 2 * (phasesPerCycle) * (numColors) )

inline printStep(rb, phase)
{
    printf("STEP: %e @ %d\n", phase, rb)
}

inline printConfig()
{
    printf("CONF: %e |\t", position);
    if
    :: robot[ROBOT_A].is_computing  -> printf("A:{%d->%d}\t", robot[ROBOT_A].color, robot[ROBOT_A].pending_color)
    :: else ->
        if
        :: robot[ROBOT_A].is_moving -> printf("A:{%d (%e)}\t", robot[ROBOT_A].color, robot[ROBOT_A].pending)
        :: else -> printf("A:{%d}\t", robot[ROBOT_A].color)
        fi
    fi;
    if
    :: robot[ROBOT_B].is_computing  -> printf("B:{%d->%d}", robot[ROBOT_B].color, robot[ROBOT_B].pending_color)
    :: else ->
        if
        :: robot[ROBOT_B].is_moving -> printf("B:{%d (%e)}", robot[ROBOT_B].color, robot[ROBOT_B].pending)
        :: else -> printf("B:{%d}", robot[ROBOT_B].color)
        fi
    fi;
    printf("\n");
}

inline reportStep(robot_id, step_name)
{
    printStep(robot_id, step_name); \
    printConfig()
}

#endif
