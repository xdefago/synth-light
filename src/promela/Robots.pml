#ifndef __ROBOTS_PML__
#define __ROBOTS_PML__

#include "Types.pml"

proctype Robot(bit me) {
    local bit other = otherRobot(me);
    local chan in = robot_in[me];
    xr in;
    local chan reply;
    local bool other_is_moving;
    local observation_t obs;
    local command_t     command;
    
    endLOOK: atomic { in ? LOOK, reply ->
        color_t seen_color = robot[other].color;
#ifndef CONSISTENCY
#  error "CONSISTENCY undefined!"
#elif (CONSISTENCY == LOOK_SAFE) || (CONSISTENCY == LOOK_REGULAR)
        {
#  if CONSISTENCY == LOOK_SAFE
            select( seen_color : (BLACK)..(MAX_COLOR));
#  elif CONSISTENCY == LOOK_REGULAR
            if
            :: seen_color = robot[other].pending_color;
            :: seen_color = robot[other].color;
            fi;
#  endif
        } unless ! robot[other].is_computing;
#endif
        obs.color.other		= seen_color;
        obs.color.me		= robot[me].color;
        obs.same_position	= position == SAME;
        obs.near_position	= (position == NEAR || position == SAME);
        other_is_moving = robot[other].is_moving;
        Algorithm(obs, command);
        if
        :: (position == SAME && ! other_is_moving)										-> robot[me].pending = STAY;
        :: (other_is_moving  && (command.move == TO_HALF || command.move == TO_OTHER))	-> robot[me].pending = MISS;
        :: else																			-> robot[me].pending = command.move
        fi;
        reportStep(me, LOOK);
        reply ! me
    }
    
    endBCOMPUTE: atomic { in ? BEGIN_COMPUTE, reply ->
        robot[me].is_computing  = true;
        robot[me].pending_color = command.new_color;
        reportStep(me, BEGIN_COMPUTE);
        reply ! me
    }
    
    endECOMPUTE: atomic { in ? END_COMPUTE, reply ->
        robot[me].is_computing = false;
        if
        :: (robot[me].color != command.new_color) ->
            eventColorChange: { robot[me].color = command.new_color }
        :: else -> skip
        fi;
        reportStep(me, END_COMPUTE);
        reply ! me
    }
    
    endBMOVE: atomic { in ? BEGIN_MOVE, reply ->
        if
        :: (robot[me].pending != STAY) ->
            eventStartMoving: {
                robot[me].is_moving = true;
            }
        :: else -> skip
        fi
        reportStep(me, BEGIN_MOVE);
        reply ! me
    }
    
    endEMOVE: atomic { in ? END_MOVE, reply ->
        if
        :: (robot[me].is_moving) ->
            local position_t new_position = position;
            assert( robot[me].pending != STAY );
            if
            :: (position == FAR) ->
                { robot[other].pending = MISS } unless (robot[other].pending == STAY);
                new_position = NEAR;
            :: (position == NEAR || position == SAME) ->
                if
                :: (robot[me].pending == MISS) -> 
                    { robot[other].pending = MISS } unless (robot[other].pending == STAY);
                    new_position = NEAR;
                :: (robot[me].pending == TO_OTHER) ->
                    { robot[other].pending = MISS } unless (robot[other].pending == STAY || position == SAME);
                    new_position = SAME;
                :: (robot[me].pending == TO_HALF) ->
                    if 
                    :: (robot[other].pending == TO_HALF) -> robot[other].pending = TO_OTHER
                    :: (robot[other].pending == STAY)    -> skip /* nothing */
                    :: else                              -> robot[other].pending = MISS
                    fi
                :: else -> assert( false )
                fi;
            fi;
            if
            :: (position != new_position) ->
                eventPositionChange:
                    position = new_position
            :: else -> skip
            fi
        :: else -> skip
        fi;
        robot[me].is_moving = false;
        robot[me].pending = STAY;
        
        reportStep(me, END_MOVE);
        reply ! me
    }

    goto endLOOK
}


#endif
