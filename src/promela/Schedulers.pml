#ifndef __SCHEDULERS_PML__
#define __SCHEDULERS_PML__

#define CENTRALIZED         (10)
#define FSYNC               (11)
#define SSYNC               (12)
#define ASYNC_SAFE          (13)
#define ASYNC_REGULAR       (14)
#define ASYNC               (15)
#define ASYNC_MOVE_SAFE     (16)
#define ASYNC_MOVE_REGULAR  (17)
#define ASYNC_MOVE_ATOMIC   (18)
#define ASYNC_LC_ATOMIC     (19)
#define ASYNC_LC_STRICT     (20)
#define ASYNC_CM_ATOMIC     (22)

#define activation_step(step_name, robot_id, reply_channel) \
    { robot_in[robot_id] ! step_name, reply_channel; \
    reply_channel ? eval(robot_id) }

#define LOOK_SAFE           (100)
#define LOOK_REGULAR        (101)
#define LOOK_ATOMIC         (102)
#define LOOK_ANY            LOOK_ATOMIC

#ifdef CONSISTENCY
#  error "CONSISTENCY already defined!"
#elif (SCHEDULER == ASYNC_SAFE) || (SCHEDULER == ASYNC_MOVE_SAFE)
#  define CONSISTENCY LOOK_SAFE
#elif (SCHEDULER == ASYNC_REGULAR) || (SCHEDULER == ASYNC_MOVE_REGULAR)
#  define CONSISTENCY LOOK_REGULAR
#else
#  define CONSISTENCY LOOK_ATOMIC
#endif


#if SCHEDULER == CENTRALIZED
#  define SCHEDULER_NAME "CENTRALIZED"
#  define Scheduler		SchedulerCentralized
#  define PHASES_PER_CYCLE (1)
#  define FAIRNESS_LIMIT      FAIR_LIMIT(PHASES_PER_CYCLE, NUM_COLORS)
proctype SchedulerCentralized() {
    local byte count_a = 0;
    local byte count_b = 0;
    local robot_t active_robot;
    local chan reply = [0] of { robot_t };
    xr reply;
    
    printf("Scheduler: Centralized\n");
    do
    :: GATHERED -> printf("*** GATHERED ***\n"); goto done
    :: else -> atomic {
        if
        :: (count_a < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_A; count_a++; count_b = 0 
        :: (count_b < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_B; count_b++; count_a = 0
        fi;
        activation_step(LOOK,		    active_robot, reply);
        activation_step(BEGIN_COMPUTE,	active_robot, reply);
        activation_step(END_COMPUTE,	active_robot, reply);
        activation_step(BEGIN_MOVE,	    active_robot, reply);
        activation_step(END_MOVE,	    active_robot, reply);
        assert( robot[active_robot].pending == STAY );
        assert( ! robot[active_robot].is_moving );
    }
    od
done:
    skip;
    printf("Scheduler FINISHED\n");
    assert ( position == SAME );
}


#elif SCHEDULER == FSYNC
#  define SCHEDULER_NAME "FSYNC"
#  define Scheduler		SchedulerFSYNC
#  define PHASES_PER_CYCLE (1)
#  define FAIRNESS_LIMIT      FAIR_LIMIT(PHASES_PER_CYCLE, NUM_COLORS)
proctype SchedulerFSYNC() {
    local chan reply = [0] of { robot_t };
    xr reply;
    printf("Scheduler: FSYNC\n");
    do
    :: GATHERED -> printf("*** GATHERED ***\n"); goto done
    :: else -> atomic {
        activation_step(LOOK, ROBOT_A, reply);
        activation_step(LOOK, ROBOT_B, reply);

        activation_step(BEGIN_COMPUTE,	ROBOT_A, reply);
        activation_step(END_COMPUTE,	ROBOT_A, reply);
        activation_step(BEGIN_MOVE, 	ROBOT_A, reply);
        activation_step(END_MOVE, 		ROBOT_A, reply);

        activation_step(BEGIN_COMPUTE,	ROBOT_B, reply);
        activation_step(END_COMPUTE,	ROBOT_B, reply);
        activation_step(BEGIN_MOVE, 	ROBOT_B, reply);
        activation_step(END_MOVE, 		ROBOT_B, reply);
        
        assert( robot[ROBOT_A].pending == STAY );
        assert( ! robot[ROBOT_A].is_moving );
        assert( robot[ROBOT_B].pending == STAY );
        assert( ! robot[ROBOT_B].is_moving );
    }
    od	
done:
    skip;
    printf("Scheduler FINISHED\n");
    assert ( position == SAME );
}


#elif SCHEDULER == SSYNC
#  define SCHEDULER_NAME "SSYNC"
#  define Scheduler		SchedulerSSYNC
#  define PHASES_PER_CYCLE (1)
#  define FAIRNESS_LIMIT      FAIR_LIMIT(PHASES_PER_CYCLE, NUM_COLORS)
proctype SchedulerSSYNC() {
    local byte count_a = 0;
    local byte count_b = 0;
    local robot_t active_robot;
    local chan reply = [0] of { robot_t };
    xr reply;
    printf("Scheduler: SSYNC\n");
    do
    :: GATHERED -> printf("*** GATHERED ***\n"); goto done
    :: else ->
        if
        :: atomic {
            count_a = 0; count_b = 0;
            activation_step(LOOK, ROBOT_A, reply);
            activation_step(LOOK, ROBOT_B, reply);
    
            activation_step(BEGIN_COMPUTE,	ROBOT_A, reply);
            activation_step(END_COMPUTE,	ROBOT_A, reply);
            activation_step(BEGIN_MOVE,		ROBOT_A, reply);
            activation_step(END_MOVE,		ROBOT_A, reply);
    
            activation_step(BEGIN_COMPUTE,	ROBOT_B, reply);
            activation_step(END_COMPUTE,	ROBOT_B, reply);
            activation_step(BEGIN_MOVE,		ROBOT_B, reply);
            activation_step(END_MOVE,		ROBOT_B, reply);
        }
        :: atomic {
            if
            :: (count_a < FAIRNESS_LIMIT) ->
                active_robot = ROBOT_A; count_a++; count_b = 0 
            :: (count_b < FAIRNESS_LIMIT) ->
                active_robot = ROBOT_B; count_b++; count_a = 0
            fi;
            activation_step(LOOK,			active_robot, reply);
            activation_step(BEGIN_COMPUTE,	active_robot, reply);
            activation_step(END_COMPUTE,	active_robot, reply);
            activation_step(BEGIN_MOVE,		active_robot, reply);
            activation_step(END_MOVE,		active_robot, reply);
        }
        fi
    od	
done:
    skip;
    printf("Scheduler FINISHED\n");
    assert ( position == SAME );
}


#elif (SCHEDULER == ASYNC) || (SCHEDULER == ASYNC_SAFE) || (SCHEDULER == ASYNC_REGULAR)
#  if SCHEDULER == ASYNC_SAFE
#    define SCHEDULER_NAME "ASYNC(safe)"
#  elif SCHEDULER == ASYNC_REGULAR
#    define SCHEDULER_NAME "ASYNC(regular)"
#  else
#    define SCHEDULER_NAME "ASYNC"
#  endif
#  define Scheduler		SchedulerASYNC
#  define PHASES_PER_CYCLE (5)
#  define FAIRNESS_LIMIT      FAIR_LIMIT(PHASES_PER_CYCLE, NUM_COLORS)
proctype SchedulerASYNC() {
    local byte count_a = 0;
    local byte count_b = 0;
    local robot_t active_robot;
    local mtype phase[2] = { LOOK, LOOK };
    local chan reply = [0] of { robot_t };
    xr reply;
    
    printf("Scheduler: ASYNC\n");
    do
    :: GATHERED -> printf("*** GATHERED ***\n"); goto done
    :: else -> atomic {
        if
        :: (count_a < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_A; count_a++; count_b = 0 
        :: (count_b < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_B; count_b++; count_a = 0
        fi;
        
        printf("Activation: robot=%d %e\n", active_robot, phase[active_robot])
        activation_step(phase[active_robot], active_robot, reply);
        if
        :: (phase[active_robot] == END_MOVE) ->
            assert( robot[active_robot].pending == STAY );
            assert( ! robot[active_robot].is_moving );
        :: else -> skip
        fi;
        phase[active_robot] = nextPhase(phase[active_robot]);
    }
    od
done:
    skip;
    printf("Scheduler FINISHED\n");
    assert ( position == SAME );
}


#elif (SCHEDULER == ASYNC_MOVE_ATOMIC) || (SCHEDULER == ASYNC_MOVE_SAFE) || (SCHEDULER == ASYNC_MOVE_REGULAR)
#  if SCHEDULER == ASYNC_SAFE
#    define SCHEDULER_NAME "ASYNC_MOVE_ATOMIC(safe)"
#  elif SCHEDULER == ASYNC_REGULAR
#    define SCHEDULER_NAME "ASYNC_MOVE_ATOMIC(regular)"
#  else
#    define SCHEDULER_NAME "ASYNC_MOVE_ATOMIC"
#  endif
#  define Scheduler		SchedulerASYNCMoveAtomic
#  define PHASES_PER_CYCLE (4)
#  define FAIRNESS_LIMIT      FAIR_LIMIT(PHASES_PER_CYCLE, NUM_COLORS)
proctype SchedulerASYNCMoveAtomic() {
    local byte count_a = 0;
    local byte count_b = 0;
    local robot_t active_robot;
    local mtype phase[2] = { LOOK, LOOK };
    local chan reply = [0] of { robot_t };
    xr reply;
    
    printf("Scheduler: ASYNC - Move Atomic\n");
    do
    :: GATHERED -> printf("*** GATHERED ***\n"); goto done
    :: else -> atomic {
        if
        :: (count_a < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_A; count_a++; count_b = 0 
        :: (count_b < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_B; count_b++; count_a = 0
        fi;
        
        printf("Activation: robot=%d %e\n", active_robot, phase[active_robot])
        activation_step(phase[active_robot], active_robot, reply);
        if
        :: (phase[active_robot] == BEGIN_MOVE) ->
            activation_step(END_MOVE, active_robot, reply);
            assert( robot[active_robot].pending == STAY );
            assert( ! robot[active_robot].is_moving );
            phase[active_robot] = LOOK;
        :: else -> phase[active_robot] = nextPhase(phase[active_robot]);
        fi;
    }
    od
done:
    skip;
    printf("Scheduler FINISHED\n");
    assert ( position == SAME );
}


#elif SCHEDULER == ASYNC_LC_ATOMIC
#  define SCHEDULER_NAME "ASYNC_LC_ATOMIC"
#  define Scheduler		SchedulerASYNCLCAtomic
#  define PHASES_PER_CYCLE (3)
#  define FAIRNESS_LIMIT      FAIR_LIMIT(PHASES_PER_CYCLE, NUM_COLORS)
proctype SchedulerASYNCLCAtomic() {
    local byte count_a = 0;
    local byte count_b = 0;
    local robot_t active_robot;
    local mtype phase[2] = { LOOK, LOOK };
    local chan reply = [0] of { robot_t };
    xr reply;
    
    printf("Scheduler: ASYNC - LC Atomic\n");
    do
    :: GATHERED -> printf("*** GATHERED ***\n"); goto done
    :: (! GATHERED && phase[ROBOT_A] == LOOK && phase[ROBOT_B] == LOOK) -> atomic {
        count_a = 0; count_b = 0;
        printf("Activation: BOTH robots in LC\n");
        activation_step(LOOK, ROBOT_A, reply);
        activation_step(LOOK, ROBOT_B, reply);
    
        activation_step(BEGIN_COMPUTE,	ROBOT_A, reply);
        activation_step(END_COMPUTE,	ROBOT_A, reply);
        activation_step(BEGIN_COMPUTE,	ROBOT_B, reply);
        activation_step(END_COMPUTE,	ROBOT_B, reply);
        
        phase[ROBOT_A] = BEGIN_MOVE;
        phase[ROBOT_B] = BEGIN_MOVE;
    }
    :: (! GATHERED) -> atomic {
        if
        :: (count_a < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_A; count_a++; count_b = 0 
        :: (count_b < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_B; count_b++; count_a = 0
        fi;
        
        printf("Activation: robot=%d %e\n", active_robot, phase[active_robot])
        activation_step(phase[active_robot], active_robot, reply);
        if
        :: (phase[active_robot] == LOOK) ->
            phase[active_robot] = BEGIN_COMPUTE;
            activation_step(phase[active_robot], active_robot, reply);
            phase[active_robot] = END_COMPUTE;
            activation_step(phase[active_robot], active_robot, reply);
        :: (phase[active_robot] == END_MOVE) ->
            assert( robot[active_robot].pending == STAY );
            assert( ! robot[active_robot].is_moving );	
        :: else -> 
        fi;
        phase[active_robot] = nextPhase(phase[active_robot]);
    }
    od
done:
    skip;
    printf("Scheduler FINISHED\n");
    assert ( position == SAME );
}


#elif SCHEDULER == ASYNC_LC_STRICT
#  define SCHEDULER_NAME "ASYNC_LC_STRICT"
#  define Scheduler		SchedulerASYNCLCStrict
#  define PHASES_PER_CYCLE (3)
#  define FAIRNESS_LIMIT      FAIR_LIMIT(PHASES_PER_CYCLE, NUM_COLORS)
proctype SchedulerASYNCLCStrict() {
    local byte count_a = 0;
    local byte count_b = 0;
    local robot_t active_robot;
    local mtype phase[2] = { LOOK, LOOK };
    local chan reply = [0] of { robot_t };
    xr reply;
    
    printf("Scheduler: ASYNC - LC Strict\n");
    do
    :: GATHERED -> printf("*** GATHERED ***\n"); goto done
    :: else -> atomic {
        if
        :: (count_a < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_A; count_a++; count_b = 0 
        :: (count_b < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_B; count_b++; count_a = 0
        fi;
        
        printf("Activation: robot=%d %e\n", active_robot, phase[active_robot])
        activation_step(phase[active_robot], active_robot, reply);
        if
        :: (phase[active_robot] == LOOK) ->
            phase[active_robot] = BEGIN_COMPUTE;
            activation_step(phase[active_robot], active_robot, reply);
            phase[active_robot] = END_COMPUTE;
            activation_step(phase[active_robot], active_robot, reply);
        :: (phase[active_robot] == END_MOVE) ->
            assert( robot[active_robot].pending == STAY );
            assert( ! robot[active_robot].is_moving );	
        :: else -> 
        fi;
        phase[active_robot] = nextPhase(phase[active_robot]);
    }
    od
done:
    skip;
    printf("Scheduler FINISHED\n");
    assert ( position == SAME );
}


#elif SCHEDULER == ASYNC_CM_ATOMIC
#  define SCHEDULER_NAME "ASYNC_CM_ATOMIC"
#  define Scheduler		SchedulerASYNCCMAtomic
#  define PHASES_PER_CYCLE (3)
#  define FAIRNESS_LIMIT      FAIR_LIMIT(PHASES_PER_CYCLE, NUM_COLORS)
proctype SchedulerASYNCCMAtomic() {
    local byte count_a = 0;
    local byte count_b = 0;
    local robot_t active_robot;
    local mtype phase[2] = { LOOK, LOOK };
    local chan reply = [0] of { robot_t };
    xr reply;
    
    printf("Scheduler: ASYNC - CM Atomic\n");
    do
    :: GATHERED -> printf("*** GATHERED ***\n"); goto done
    :: else -> atomic {
        if
        :: (count_a < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_A; count_a++; count_b = 0 
        :: (count_b < FAIRNESS_LIMIT) ->
            active_robot = ROBOT_B; count_b++; count_a = 0
        fi;
        
        printf("Activation: robot=%d %e\n", active_robot, phase[active_robot])
        activation_step(phase[active_robot], active_robot, reply);
        if
        :: (phase[active_robot] == BEGIN_COMPUTE) ->
            phase[active_robot] = END_COMPUTE;
            activation_step(phase[active_robot], active_robot, reply);
            phase[active_robot] = BEGIN_MOVE;
            activation_step(phase[active_robot], active_robot, reply);
            phase[active_robot] = END_MOVE;
            activation_step(phase[active_robot], active_robot, reply);
            assert( robot[active_robot].pending == STAY );
            assert( ! robot[active_robot].is_moving );	
        :: else -> 
        fi;
        phase[active_robot] = nextPhase(phase[active_robot]);
    }
    od
done:
    skip;
    printf("Scheduler FINISHED\n");
    assert ( position == SAME );
}


#else
#  error "No scheduler defined. Define SCHEDULER"
#endif

#endif
