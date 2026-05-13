# i-rs-remind Examples

## Basic Usage

### Creating Reminders

```bash
# Simple date reminder
i-rs-remind add dentist-appointment 2025-03-15 --title "Dental Checkup" --tag health --tag important

# Reminder with time
i-rs-remind add team-meeting 2025-06-15 14:00 --title "Weekly Team Sync" --tag work --tag recurring

# Birthday reminder
i-rs-remind add moms-birthday 2025-08-20 --title "Mom's Birthday" --tag personal --tag family --content "Get a gift"

# Project deadline
i-rs-remind add project-deadline 2025-07-01 18:00 --title "Project Submission" --tag work --tag deadline --content "Submit final report"
```

### Managing Reminders

```bash
# List all reminders
i-rs-remind list

# List work reminders
i-rs-remind list --tag work

# List personal reminders
i-rs-remind list --tag personal

# Get reminder details
i-rs-remind get team-meeting
```

### Completing Reminders

```bash
# Mark as done
i-rs-remind done team-meeting

# After marking, reminder stays in list but shows as done
i-rs-remind list
```

### Updating Reminders

```bash
# Reschedule meeting
i-rs-remind update team-meeting --event-date 2025-06-22 14:00

# Change title
i-rs-remind update project-deadline --title "Project Submission - FINAL"

# Add more content
i-rs-remind update project-deadline --content "Submit final report" --content "Email to manager"
```

### Deleting Reminders

```bash
# Delete completed or unwanted reminders
i-rs-remind delete old-reminder
i-rs-remind delete test-event
```

## Organized Reminders

### Work Reminders

```bash
# Regular meetings
i-rs-remind add weekly-standup 2025-01-13 09:00 --title "Monday Standup" --tag work --tag meeting --tag weekly
i-rs-remind add sprint-planning 2025-01-15 10:00 --title "Sprint Planning" --tag work --tag meeting --tag agile

# Project milestones
i-rs-remind add design-review 2025-02-01 14:00 --title "Design Review" --tag work --tag project --tag review
i-rs-remind add code-freeze 2025-03-15 18:00 --title "Code Freeze" --tag work --tag project --tag milestone

# Deadlines
i-rs-remind add q1-deadline 2025-03-31 --title "Q1 Goals" --tag work --tag deadline --tag goals
```

### Personal Reminders

```bash
# Birthdays
i-rs-remind add john-bday 2025-05-20 --title "John's Birthday" --tag personal --tag birthday --tag friend
i-rs-remind add anniv 2025-07-15 --title "Anniversary" --tag personal --tag family --content "Dinner reservation"

# Appointments
i-rs-remind add doctor-visit 2025-02-10 11:00 --title "Annual Checkup" --tag personal --tag health
i-rs-remind add car-service 2025-04-01 --title "Car Maintenance" --tag personal --tag car

# Events
i-rs-remind add conference 2025-09-15 --title "Tech Conference" --tag personal --tag event --tag learning
```

### Recurring Reminders

```bash
# Weekly
i-rs-remind add gym-mon 2025-01-13 07:00 --title "Gym" --tag health --tag fitness --content "Leg day"
i-rs-remind add gym-wed 2025-01-15 07:00 --title "Gym" --tag health --tag fitness --content "Upper body"

# Monthly
i-rs-remind add rent-due 2025-02-01 --title "Rent Due" --tag bills --tag monthly --content "$1500"
i-rs-remind add savings-transfer 2025-02-05 --title "Transfer to Savings" --tag finance --tag monthly
```

## Real-World Workflows

### Daily Standup

```bash
# Morning: check upcoming reminders
i-rs-remind list

# Mark completed items
i-rs-remind done meeting-1
i-rs-remind done standup-yesterday
```

### Weekly Review

```bash
# Review all work reminders
i-rs-remind list --tag work

# Mark done
i-rs-remind done completed-task-1

# Reschedule if needed
i-rs-remind update delayed-task --event-date 2025-01-20
```

### Planning

```bash
# Add new project milestones
i-rs-remind add kickoff 2025-01-15 10:00 --title "Project Kickoff" --tag project-alpha --tag meeting
i-rs-remind add review-1 2025-02-15 14:00 --title "First Review" --tag project-alpha --tag review
i-rs-remind add final-delivery 2025-03-31 18:00 --title "Final Delivery" --tag project-alpha --tag deadline
```