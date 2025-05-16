
#ifndef ALGO
#  error "define ALGO"
#  define ALGO      ALGO_OPTIMAL
#endif
#ifndef SCHEDULER
#  error "define SCHEDULER"
#  define SCHEDULER ASYNC
#endif
#ifndef MOVEMENT
#  define MOVEMENT  NON_RIGID
#endif


#define MAX_CONSECUTIVE		(3)

#include "Types.pml"

#include "Algorithms.pml"
#include "Schedulers.pml"
#include "Robots.pml"


ltl gathering {
    <> [] (position == SAME)
}

init {
    printf("SCHEDULER:");
    printf(SCHEDULER_NAME);
    printf("\n");
    printf("ALGORITHM:");
    printf(ALGO_NAME);
    printf("\n");
    /* Initial colors (non-deterministic selection)  */
#ifndef QUASISS
    int i;
    for (i in robot) {
        int col;
        select ( col : BLACK .. (MAX_COLOR) );
        robot[i].color = col;
    }
#else
    int col;
    select ( col : BLACK .. (MAX_COLOR) );
    int i;
    for (i in robot) {
        robot[i].color = col;
    }
#endif
    /* Initial position (non-deterministic selection) */
#if MOVEMENT == RIGID
    int pos;
    select ( pos : (NEAR) .. (SAME) );
    position = pos;
#else
    int pos;
    select ( pos : (FAR) .. (SAME) );
    position = pos;
#endif

    printConfig();
    
    atomic {
        run Robot(ROBOT_A);
        run Robot(ROBOT_B);
        run Scheduler();
    }
}

