# Contributing to Astra Landing Page

Thanks for your interest in contributing! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/astra-landing.git`
3. Create a branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Test locally: `npm run dev`
6. Commit your changes: `git commit -m "Add your feature"`
7. Push to your fork: `git push origin feature/your-feature-name`
8. Open a Pull Request

## Development Guidelines

### Code Style

- Use TypeScript for all new files
- Follow the existing code structure
- Use Tailwind CSS for styling
- Keep components small and focused
- Use meaningful variable and function names

### Design Guidelines

- Maintain the clean, minimal aesthetic
- Use sharp corners (no rounded borders)
- Stick to the cream/white/black/gray color palette
- Use Cabinet Grotesk for headings (medium weight)
- Use DM Sans for body text
- Use Space Grotesk for logo text

### Component Guidelines

- Create reusable components when possible
- Keep components in appropriate directories
- Use client components ('use client') only when necessary
- Prefer server components by default

### Commit Messages

- Use clear, descriptive commit messages
- Start with a verb (Add, Fix, Update, Remove, etc.)
- Keep the first line under 50 characters
- Add details in the body if needed

Example:
```
Add dashboard sidebar navigation

- Created DashboardSidebar component
- Added navigation menu with icons
- Implemented active state highlighting
```

## Pull Request Process

1. Update the README.md if you add new features
2. Ensure your code builds without errors: `npm run build`
3. Test your changes thoroughly
4. Describe your changes clearly in the PR description
5. Link any related issues

## Reporting Issues

- Use the GitHub issue tracker
- Describe the issue clearly
- Include steps to reproduce
- Add screenshots if relevant
- Mention your environment (OS, browser, Node version)

## Questions?

Feel free to open an issue for any questions or discussions.

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Help others learn and grow

Thank you for contributing!
