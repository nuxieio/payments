# Working with the Payments App as a Git Subtree

The Payments app is maintained as a separate repository at [github.com/nuxieio/payments](https://github.com/nuxieio/payments) but is included in this monorepo as a Git subtree.

## What is a Git Subtree?

A Git subtree allows you to include a copy of an external repository within your own repository. This is different from Git submodules in that:

- The subtree is a copy of the external repository, not a reference to it
- Changes to the subtree are committed to your repository
- You don't need to initialize or update the subtree after cloning the repository

## Working with the Payments App

### Making Changes

When you make changes to the Payments app within this monorepo, you can commit them as you would any other changes. The changes will be committed to the monorepo, but not to the external Payments repository.

### Pulling Changes from the External Repository

To pull changes from the external Payments repository:

```bash
git subtree pull --prefix=apps/payments payments main
```

### Pushing Changes to the External Repository

If you want to push your changes to the external Payments repository:

```bash
git subtree push --prefix=apps/payments payments main
```

### Resolving Conflicts

If you encounter conflicts when pulling changes from the external repository, you'll need to resolve them as you would any other Git conflicts.

## Best Practices

1. **Coordinate with the Payments Team**: If you're making changes to the Payments app, coordinate with the team responsible for it to avoid conflicts.

2. **Pull Before Push**: Always pull changes from the external repository before pushing your changes to avoid conflicts.

3. **Keep Changes Small**: Try to keep your changes small and focused to minimize the risk of conflicts.

4. **Document Your Changes**: Document your changes in the commit messages to make it easier for others to understand what you've done. 
