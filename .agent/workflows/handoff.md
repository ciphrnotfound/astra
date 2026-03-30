---
description: How to use Astra as a proactive Tech Lead for your IDE.
---
# Astra Tech Lead: Proactive Delegation Workflow

Use this workflow to have Astra manage your IDE agents (Windsurf/Cursor) autonomously.

// turbo
1. **Talk to Astra Globally**: chat with the Astra CLI to set a goal.
   ```bash
   astra "I want to [your goal here]"
   ```

2. **Handover to IDE**: Open your IDE (Windsurf/Cursor).
   
3. **Trigger Delegation**: Simply ask your IDE agent:
   > "Astra, what's next?"
   
   Astra will automatically retrieve your previous conversation and give the IDE a 3-step structured plan.

4. **Execution**: Your IDE agent works on the task.

5. **Astra Review**: Once the agent is done, Astra moves the task to "Review" state.
   The agent will then ask Astra again, and Astra will issue a "Production Readiness Review" task to verify the code quality.

6. **Final Approval**: Astra confirms the job is done and restores the health scores.
