import { Input } from '@fluentui/react-components';
import { SearchRegular } from '@fluentui/react-icons';

export function TopHeader() {
  return (
    <header className="top-header">
      <Input
        className="top-header-search"
        contentBefore={<SearchRegular />}
        placeholder="Search emails, codes, or senders..."
        appearance="outline"
      />
    </header>
  );
}
