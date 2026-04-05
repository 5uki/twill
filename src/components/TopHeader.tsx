import { Input } from '@fluentui/react-components';
import { SearchRegular } from '@fluentui/react-icons';

export function TopHeader() {
  return (
    <header className="top-header">
      <Input
        className="top-header-search"
        contentBefore={<SearchRegular />}
        placeholder="搜索邮件、验证码或发件人"
        appearance="outline"
      />
    </header>
  );
}
