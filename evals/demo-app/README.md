# dagr demo app

Triple-duty fixture: onboarding target for the 60-second `prove` tour, an
eval task source, and CI smoke input. It intentionally contains:

- a UI→DB boundary violation (guard must flag it),
- an import of a module that `demo-deletion.sh` removes (review-diff must BLOCK).

## Try it

```bash
dagr prove                       # receipt with FINDINGS PRESENT
dagr review-diff HEAD~1 HEAD     # run after demo-deletion.sh
```
