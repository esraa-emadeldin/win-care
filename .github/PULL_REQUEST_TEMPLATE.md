## What and why

<!-- One topic per pull request. Link the issue: Closes #123 -->

## Checklist

- [ ] `npm run check` and `npm run test:e2e` pass
- [ ] A test fails without this change and passes with it
- [ ] Scanning still writes nothing; every change goes through `make_plan` and `apply`
- [ ] Nothing can be deleted permanently or outside the user's profile by accident
- [ ] No telemetry, no new network use, no admin rights
- [ ] User-facing text is short and plain; screenshots attached for UI changes
- [ ] Conventional commit messages
