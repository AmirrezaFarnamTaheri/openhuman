import type { Locale } from '../types';
import ar from './ar';
import bn from './bn';
import de from './de';
import en, { type ModelManagementMessages } from './en';
import es from './es';
import fr from './fr';
import hi from './hi';
import id from './id';
import it from './it';
import ja from './ja';
import ko from './ko';
import pl from './pl';
import pt from './pt';
import ru from './ru';
import tr from './tr';
import zhCN from './zh-CN';

/** Complete feature messages; missing translations are a compile-time error. */
export const modelManagementTranslations = {
  en,
  es,
  fr,
  de,
  pt,
  it,
  tr,
  ru,
  pl,
  id,
  ja,
  ko,
  'zh-CN': zhCN,
  ar,
  hi,
  bn,
} satisfies Record<Locale, ModelManagementMessages>;
