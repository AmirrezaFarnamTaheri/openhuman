import type { ModelManagementMessages } from './en';

const messages = {
  'feasibility.contextTooSmall': 'কনটেক্সটের আকার ইনপুট ও আউটপুটের মোট টোকেনের চেয়ে ছোট',
  'feasibility.visionUnavailable': 'সংরক্ষিত মেটাডেটা অনুযায়ী ছবি বোঝার সুবিধা সমর্থিত নয়',
  'feasibility.overBudget': 'আনুমানিক খরচ বাজেটের চেয়ে বেশি',
  'feasibility.unknownContext': 'কনটেক্সটের আকার অজানা',
  'feasibility.unknownVision': 'ছবি বোঝার সক্ষমতা অজানা',
  'feasibility.unknownPrice': 'মূল্য দেওয়া নেই, শূন্য অথবা অবৈধ',
  'feasibility.aria': 'মডেলের উপযুক্ততার প্রাকদর্শন',
  'feasibility.title': 'মডেলের উপযুক্ততা ও খরচের পূর্বরূপ',
  'feasibility.advisory':
    'এই প্রাকদর্শন সংরক্ষিত মেটাডেটার ওপর ভিত্তি করে তৈরি। বর্তমান মূল্য, কোটা, কী বা প্রাপ্যতা যাচাই করা হয়নি। এটি শুধু পরামর্শ দেয় এবং কোনো কাজের রাউটিং পরিবর্তন করে না।',
  'feasibility.input': 'আনুমানিক ইনপুট টোকেন',
  'feasibility.cached': 'ক্যাশে রাখা ইনপুট টোকেন (ইনপুটের অংশ)',
  'feasibility.output': 'আনুমানিক আউটপুট টোকেন',
  'feasibility.budget': 'খরচের সীমা (USD, ঐচ্ছিক)',
  'feasibility.budgetAria': 'মার্কিন ডলারে আনুমানিক খরচের সীমা',
  'feasibility.noLimit': 'কোনো সীমা নেই',
  'feasibility.requireVision': 'ছবি বোঝার সক্ষমতা আবশ্যক',
  'feasibility.invalidCached':
    'ক্যাশে রাখা ইনপুটের টোকেনসংখ্যা মোট ইনপুট টোকেনের চেয়ে বেশি হতে পারবে না।',
  'feasibility.invalidCounts':
    'টোকেনসংখ্যা শূন্য বা তার চেয়ে বড় পূর্ণসংখ্যায় লিখুন। ক্যাশে রাখা ইনপুট মোট ইনপুটেরই অংশ। চাইলে খরচের সীমাও দিতে পারেন; তা ঋণাত্মক হতে পারবে না।',
  'feasibility.fitCount': 'সংরক্ষিত মেটাডেটা অনুযায়ী উপযুক্ত',
  'feasibility.unknownCount': 'অজানা',
  'feasibility.excludedCount': 'দেওয়া শর্তের কারণে বাদ পড়েছে',
  'feasibility.search': 'উপযুক্ততার ফলাফলে খুঁজুন',
  'feasibility.filter': 'উপযুক্ততার অবস্থা অনুযায়ী ফিল্টার',
  'feasibility.all': 'সব অবস্থা',
  'feasibility.fits': 'সংরক্ষিত তথ্য অনুযায়ী উপযুক্ত',
  'feasibility.unknown': 'উপযুক্ততা অজানা',
  'feasibility.excluded': 'বাদ দেওয়া হয়েছে',
  'feasibility.fitDescription': 'সংরক্ষিত মেটাডেটার সঙ্গে মেলে; প্রাপ্যতা যাচাই করা হয়নি',
  'feasibility.excludedDescription': 'শর্তের কারণে বাদ পড়েছে',
  'feasibility.costUnknown': 'খরচ অজানা',
  'feasibility.empty': 'মূল্যায়নের জন্য সংরক্ষিত মডেল নেই।',
  'feasibility.noMatch': 'এই ফিল্টারের সাথে কোনো মডেল মেলেনি।',
  'feasibility.showing': 'দেখানো হচ্ছে',
  'feasibility.of': 'এর মধ্যে',
  'feasibility.matching': 'মিল পাওয়া মডেল এন্ট্রি',
  'feasibility.more': 'আরও মডেল দেখান',
  'fleet.reauthRequired':
    'এন্ডপয়েন্ট বা প্রমাণীকরণের পদ্ধতি পরিবর্তনের আগে প্রোভাইডার এডিটরে API কী আবার দিন।',
  'fleet.groupSyncHintSafe':
    'সংরক্ষিত প্রোভাইডার এন্ডপয়েন্ট ডিফল্টভাবে অপরিবর্তিত থাকে। পরিবর্তনগুলো পর্যালোচনা করে স্পষ্টভাবে অনুমোদন দিন। যেসব প্রোভাইডারের ক্রেডেনশিয়াল সংরক্ষিত আছে, সেগুলো এডিটরে নতুন কী দিয়ে আপডেট করতে হবে।',
  'fleet.approveProviderSync': 'প্রোভাইডারের এন্ডপয়েন্ট ও প্রমাণীকরণ বদল অনুমোদন করুন',
  'fleet.previewProviderChanges': 'প্রোভাইডার পরিবর্তনের পূর্বরূপ',
  'fleet.providerNeedsKey': 'কী আবার লিখতে হবে',
  'fleet.presetHintSafe':
    'সংরক্ষিত মডেল মেটাডেটা পরিবর্তন না করেই অনুপস্থিত মডেল প্রিসেট আমদানি করুন। ছবি বোঝার সক্ষমতা, কনটেক্সট ও মূল্যের বিদ্যমান তথ্য আপডেট করা ঐচ্ছিক এবং এতে হাতে করা পরিবর্তন মুছে যেতে পারে। রাউটিং ও ক্রেডেনশিয়াল কখনো পরিবর্তন করা হয় না।',
  'fleet.approveMetadataRefresh': 'সংরক্ষিত মডেল তথ্য প্রতিস্থাপন অনুমোদন করুন',
  'fleet.previewModelChanges': 'মডেল পরিবর্তনের পূর্বরূপ',
  'fleet.refreshLabel': 'সংরক্ষিত মেটাডেটা হালনাগাদ করুন',
  'fleet.addLabel': 'অনুপস্থিত প্রিসেট যোগ করুন',
  'fleet.healthUnavailable': 'মডেলসমূহের অবস্থা পাওয়া যাচ্ছে না।',
  'fleet.selectProvidersError': 'এক বা একাধিক প্রোভাইডার বেছে নিন।',
  'fleet.imageEndpointError':
    'ইমেজ এন্ডপয়েন্টে HTTPS অথবা লুপব্যাক ঠিকানায় HTTP ব্যবহার করতে হবে।',
  'fleet.chooseGroupError': 'একটি প্রোভাইডার গ্রুপ বেছে নিন।',
  'fleet.modelImageEndpointError':
    'HTTPS অথবা লুপব্যাক ঠিকানায় HTTP ব্যবহার করে এমন একটি ইমেজ এন্ডপয়েন্ট দিন।',
  'fleet.selectModelsError': 'এক বা একাধিক মডেল বেছে নিন।',
  'fleet.contextError':
    'কনটেক্সট উইন্ডোর আকার 0 থেকে 100,000,000-এর মধ্যে একটি পূর্ণসংখ্যা হতে হবে।',
  'fleet.selectPatchError': 'আপডেট করার জন্য অন্তত একটি ফিল্ড বেছে নিন।',
  'fleet.noKeyUsage': 'এই সেশনে ইনফারেন্স কী ব্যবহারের কোনো রেকর্ড নেই।',
  'fleet.selectProvider': 'প্রোভাইডার বেছে নিন',
  'fleet.selected': 'বেছে নেওয়া হয়েছে',
  'fleet.groupImageAria': 'গ্রুপের ইমেজ API-এর মূল URL',
  'fleet.rotationAria': 'একাধিক টোকেন থেকে বাছাইয়ের কৌশল',
  'fleet.imageGroupAria': 'ইমেজ এন্ডপয়েন্টের প্রোভাইডার গ্রুপ',
  'fleet.imageModelIdAria': 'ইমেজ এন্ডপয়েন্টের মডেল আইডি',
  'fleet.modelImageAria': 'মডেলের ইমেজ API-এর মূল URL',
  'fleet.searchAria': 'মডেল প্রিসেট ছেঁকে নিন',
  'fleet.selectModel': 'মডেল বেছে নিন',
  'fleet.visionLabel': 'ছবি বোঝা',
  'fleet.keepUnchanged': 'অপরিবর্তিত রাখুন',
  'fleet.enabled': 'সক্রিয়',
  'fleet.disabled': 'নিষ্ক্রিয়',
  'fleet.leaveBlank': 'বিদ্যমান মান রাখতে ফাঁকা রাখুন',
  'fleet.bulkContextAria': 'একসঙ্গে আপডেট করার কনটেক্সট টোকেনসংখ্যা',
  'fleet.prodexImageError':
    'Prodex ইমেজ এন্ডপয়েন্টে HTTPS অথবা লুপব্যাক ঠিকানায় HTTP ব্যবহার করতে হবে এবং URL-এ ক্রেডেনশিয়াল থাকা যাবে না।',
  'fleet.prodexModelImageError':
    'Prodex মডেল ইমেজ এন্ডপয়েন্টে HTTPS অথবা লুপব্যাক ঠিকানায় HTTP ব্যবহার করতে হবে এবং URL-এ ক্রেডেনশিয়াল থাকা যাবে না।',
} satisfies ModelManagementMessages;

export default messages;
